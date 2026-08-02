use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    sync::mpsc,
    thread,
    time::Duration,
};

#[derive(Debug)]
struct CapturedRequest {
    method: String,
    path: String,
    authorization: Option<String>,
    body: String,
}

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_llmpulse")
}

fn mock_server(
    status: u16,
    response_body: &'static str,
) -> (
    String,
    mpsc::Receiver<CapturedRequest>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let count = stream.read(&mut buffer).unwrap_or(0);
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..count]);
            if request_is_complete(&bytes) {
                break;
            }
        }
        let raw = String::from_utf8(bytes).unwrap();
        let (headers, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
        let mut lines = headers.lines();
        let request_line = lines.next().unwrap();
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().unwrap().to_owned();
        let path = request_parts.next().unwrap().to_owned();
        let authorization = lines.find_map(|line| {
            line.strip_prefix("authorization: ")
                .or_else(|| line.strip_prefix("Authorization: "))
                .map(str::to_owned)
        });
        sender
            .send(CapturedRequest {
                method,
                path,
                authorization,
                body: body.to_owned(),
            })
            .unwrap();

        let reason = if status == 201 { "Created" } else { "OK" };
        write!(
            stream,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        )
        .unwrap();
    });
    (format!("http://{address}/api/v1"), receiver, handle)
}

fn request_is_complete(bytes: &[u8]) -> bool {
    let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = String::from_utf8_lossy(&bytes[..header_end]);
    let content_length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    });
    bytes.len() >= header_end + 4 + content_length.unwrap_or(0)
}

fn run(base_url: &str, args: &[&str]) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .args(["--api-key", "test_key", "--base-url", base_url])
        .output()
        .unwrap()
}

#[test]
fn reports_the_rust_cli_version() {
    let output = Command::new(binary()).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "llmpulse 2.0.0"
    );
}

#[test]
fn ping_uses_bearer_authentication() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"status":"ok"}"#);
    let output = run(&base_url, &["ping"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"status\":\"ok\""));
    let request = receiver.recv().unwrap();
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/api/v1/ping");
    assert_eq!(request.authorization.as_deref(), Some("Bearer test_key"));
    handle.join().unwrap();
}

#[test]
fn creates_prompts_with_a_json_body() {
    let (base_url, receiver, handle) = mock_server(201, r#"{"created":2}"#);
    let output = run(
        &base_url,
        &[
            "prompts",
            "create",
            "best crm",
            "top crm tools",
            "--country-code",
            "US",
            "--language-code",
            "en",
            "--project-id",
            "42",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let request = receiver.recv().unwrap();
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/api/v1/prompts");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(body["project_id"], 42);
    assert_eq!(
        body["prompts"],
        serde_json::json!(["best crm", "top crm tools"])
    );
    handle.join().unwrap();
}

#[test]
fn agent_bot_catalog_does_not_require_a_project() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"bots":[{"slug":"gptbot"}]}"#);
    let output = run(&base_url, &["dimensions", "agent-bots"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("gptbot"));
    assert_eq!(
        receiver.recv().unwrap().path,
        "/api/v1/dimensions/agent_bots"
    );
    handle.join().unwrap();
}

#[test]
fn refuses_remote_plain_http_urls() {
    let output = Command::new(binary())
        .args([
            "ping",
            "--api-key",
            "test_key",
            "--base-url",
            "http://example.com/api/v1",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Refusing to use insecure base URL"));
}

#[test]
fn writes_configuration_with_restricted_permissions() {
    let directory = tempfile::tempdir().unwrap();
    let output = Command::new(binary())
        .args(["config", "set", "default_project_id", "77"])
        .env("LLMPULSE_CONFIG_DIR", directory.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("config.json")).unwrap())
            .unwrap();
    assert_eq!(value["default_project_id"], 77);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(directory.path().join("config.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}
