//! Holds the CLI to the bundled OpenAPI contract per endpoint, not just per
//! parameter name. Every leaf command is run against a local server that
//! records the request, once with only its required arguments and once per
//! optional flag, and the query parameters each documented GET receives are
//! compared with the ones it documents.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    process::Command,
    thread,
    time::Duration,
};

use clap::{ArgAction, CommandFactory};
use llmpulse_cli::cli::Cli;
use serde_json::Value;

// Query parameters the CLI deliberately never sends, with the reason.
const UNSUPPORTED_QUERY_PARAMS: &[(&str, &str)] = &[(
    "output",
    "the CLI formats json, table and csv itself, through the global --format flag",
)];

// Documented parameters the server does not read, so the CLI rightly leaves
// them out. They are contract errors to fix in config/openapi (LLM-3278); this
// list only shrinks.
const NOT_READ_BY_THE_SERVER: &[(&str, &str)] = &[
    ("project_id", "/dimensions/agent_bots"),
    ("project_id", "/dimensions/competitors/{id}"),
];

// Global flags every command accepts; each is exercised once, on a command
// that sends it, by the per-flag runs below.
const GLOBAL_ARGS: &[&str] = &[
    "api_key",
    "base_url",
    "project_id",
    "output",
    "verbose",
    "all",
    "help",
    "version",
];

struct Recorded {
    method: String,
    path: String,
    query: BTreeSet<String>,
}

fn record_one_request(args: &[String], workdir: &Path) -> Option<Recorded> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        listener.set_nonblocking(false).unwrap();
        let (mut stream, _) = listener.accept().ok()?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 8192];
        while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = stream.read(&mut buffer).unwrap_or(0);
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..count]);
        }
        let head = String::from_utf8_lossy(&bytes).to_string();
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        );
        let request_line = head.lines().next()?.to_owned();
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_owned();
        let target = parts.next()?.to_owned();
        let (path, query) = target.split_once('?').unwrap_or((&target, ""));
        let query = query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let name = pair.split('=').next().unwrap();
                name.replace("%5B%5D", "").replace("[]", "")
            })
            .collect();
        Some(Recorded {
            method,
            path: path.trim_start_matches("/api/v1").to_owned(),
            query,
        })
    });

    let base_url = format!("http://{address}/api/v1");
    // Run inside the temp dir (export writes files) and straight to the local
    // recorder, whatever proxy the machine running the tests has configured.
    let mut child = Command::new(env!("CARGO_BIN_EXE_llmpulse"))
        .current_dir(workdir)
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env_remove("HTTP_PROXY")
        .env_remove("HTTPS_PROXY")
        .env_remove("ALL_PROXY")
        .env_remove("http_proxy")
        .env_remove("https_proxy")
        .env_remove("all_proxy")
        .args(args)
        .args([
            "--api-key",
            "test_key",
            "--base-url",
            &base_url,
            "--project-id",
            "1",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while child.try_wait().unwrap().is_none() && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    let _ = child.kill();
    let _ = child.wait();
    // Unblock the accept when the command never connected.
    let _ = std::net::TcpStream::connect(address);
    server
        .join()
        .unwrap_or(None)
        .filter(|recorded| !recorded.method.is_empty())
}

fn sample_value(arg: &clap::Arg, workdir: &Path) -> String {
    if let Some(value) = arg.get_possible_values().first() {
        return value.get_name().to_owned();
    }
    let id = arg.get_id().as_str();
    if id.contains("file") || id == "body" || id.ends_with("_json") {
        let file = workdir.join(format!("{id}.json"));
        fs::write(&file, "{}").unwrap();
        return file.to_string_lossy().into_owned();
    }
    if id == "from" || id == "to" || id.ends_with("_date") {
        return "2026-01-01".to_owned();
    }
    if id.contains("sha") {
        return "a".repeat(64);
    }
    if id.contains("url") {
        return "https://example.com".to_owned();
    }
    "1".to_owned()
}

fn leaves(
    command: &clap::Command,
    prefix: Vec<String>,
    found: &mut Vec<(Vec<String>, clap::Command)>,
) {
    let subcommands: Vec<_> = command.get_subcommands().collect();
    if subcommands.is_empty() {
        found.push((prefix, command.clone()));
        return;
    }
    for sub in subcommands {
        let mut path = prefix.clone();
        path.push(sub.get_name().to_owned());
        leaves(sub, path, found);
    }
}

fn arguments_for(arg: &clap::Arg, workdir: &Path) -> Vec<String> {
    let value = sample_value(arg, workdir);
    match arg.get_long() {
        Some(long) => match arg.get_action() {
            ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count => {
                vec![format!("--{long}")]
            }
            _ => vec![format!("--{long}"), value],
        },
        None => vec![value],
    }
}

fn documented_get_operations() -> BTreeMap<String, BTreeSet<String>> {
    let document: Value = serde_json::from_str(
        &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("openapi.json")).unwrap(),
    )
    .unwrap();
    let components = &document["components"]["parameters"];
    let mut operations = BTreeMap::new();
    for (path, entry) in document["paths"].as_object().unwrap() {
        let Some(operation) = entry.get("get") else {
            continue;
        };
        let tags = operation["tags"].as_array().cloned().unwrap_or_default();
        if tags
            .iter()
            .any(|tag| matches!(tag.as_str(), Some("MCP" | "OAuth 2.1")))
        {
            continue;
        }
        let names = operation["parameters"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|parameter| match parameter["$ref"].as_str() {
                Some(reference) => components[reference.rsplit('/').next().unwrap()].clone(),
                None => parameter.clone(),
            })
            .filter(|parameter| parameter["in"] == "query")
            .map(|parameter| parameter["name"].as_str().unwrap().to_owned())
            .collect();
        operations.insert(path.clone(), names);
    }
    operations
}

fn matches_template(template: &str, path: &str) -> bool {
    let left: Vec<_> = template.split('/').collect();
    let right: Vec<_> = path.split('/').collect();
    left.len() == right.len()
        && left
            .iter()
            .zip(&right)
            .all(|(expected, actual)| expected.starts_with('{') || expected == actual)
}

#[test]
fn every_documented_get_parameter_reaches_its_own_endpoint() {
    let workdir = tempfile::tempdir().unwrap();
    let operations = documented_get_operations();
    let exempt: BTreeSet<_> = UNSUPPORTED_QUERY_PARAMS
        .iter()
        .map(|(name, _)| *name)
        .collect();

    let mut all = Vec::new();
    leaves(&Cli::command(), Vec::new(), &mut all);

    let mut received: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (path, command) in all {
        let arguments: Vec<_> = command
            .get_arguments()
            .filter(|arg| !GLOBAL_ARGS.contains(&arg.get_id().as_str()))
            .cloned()
            .collect();
        let required: Vec<String> = arguments
            .iter()
            .filter(|arg| arg.is_required_set())
            .flat_map(|arg| arguments_for(arg, workdir.path()))
            .collect();
        let mut runs = vec![required.clone()];
        for arg in arguments.iter().filter(|arg| !arg.is_required_set()) {
            let mut run = required.clone();
            run.extend(arguments_for(arg, workdir.path()));
            runs.push(run);
        }
        runs.push(
            vec![
                "--page".to_owned(),
                "2".to_owned(),
                "--per-page".to_owned(),
                "5".to_owned(),
            ]
            .into_iter()
            .chain(required.clone())
            .collect(),
        );

        for run in runs {
            let mut args = path.clone();
            args.extend(run);
            let Some(recorded) = record_one_request(&args, workdir.path()) else {
                continue;
            };
            if recorded.method != "GET" {
                continue;
            }
            let Some((template, _)) = operations
                .iter()
                .find(|(template, _)| matches_template(template, &recorded.path))
            else {
                continue;
            };
            received
                .entry(template.clone())
                .or_default()
                .extend(recorded.query);
        }
    }

    // The other direction (a flag sent where the endpoint ignores it) is not
    // checked: the listing commands share one filter set on purpose.
    let mut unreached = Vec::new();
    for (template, documented) in &operations {
        let sent = received.get(template).cloned().unwrap_or_default();
        for name in documented.difference(&sent) {
            let known = NOT_READ_BY_THE_SERVER.contains(&(name.as_str(), template.as_str()));
            if !exempt.contains(name.as_str()) && !known {
                unreached.push(format!("{name} (GET {template})"));
            }
        }
    }
    let stale: Vec<_> = NOT_READ_BY_THE_SERVER
        .iter()
        .filter(|(name, template)| {
            received
                .get(*template)
                .is_some_and(|sent| sent.contains(*name))
        })
        .collect();

    assert!(
        unreached.is_empty(),
        "no CLI command sends these documented query parameters to their endpoint. Wire each one into the \
         command that calls it, or add it to UNSUPPORTED_QUERY_PARAMS with the reason: {unreached:#?}"
    );
    assert!(
        stale.is_empty(),
        "the CLI now sends these; remove them from NOT_READ_BY_THE_SERVER: {stale:#?}"
    );
}
