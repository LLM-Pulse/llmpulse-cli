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
        "llmpulse 2.2.0"
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
fn edits_llms_txt_report_content_with_a_json_body() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"id":7,"changed_files":["llms_txt"]}"#);
    let output = run(
        &base_url,
        &[
            "reports",
            "technical-geo-update-content",
            "7",
            "--report-type",
            "llms_txt",
            "--content-version",
            "1790683200",
            "--edits",
            r##"{"llms_txt":"# example.com\n"}"##,
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
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.path, "/api/v1/technical_geo_reports/7/content");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(body["project_id"], 42);
    assert_eq!(body["report_type"], "llms_txt");
    assert_eq!(body["content_version"], "1790683200");
    assert_eq!(
        body["edits"],
        serde_json::json!({ "llms_txt": "# example.com\n" })
    );
    handle.join().unwrap();
}

#[test]
fn rejects_llms_txt_edits_that_are_not_a_json_object() {
    let output = run(
        "http://127.0.0.1:9/api/v1",
        &[
            "reports",
            "technical-geo-update-content",
            "7",
            "--report-type",
            "llms_txt",
            "--content-version",
            "1790683200",
            "--edits",
            r#"["llms_txt"]"#,
            "--project-id",
            "42",
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--edits must be a JSON object"));
}

#[test]
fn reverts_llms_txt_report_content() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"id":7}"#);
    let output = run(
        &base_url,
        &[
            "reports",
            "technical-geo-revert-content",
            "7",
            "--report-type",
            "llms_txt",
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
    assert_eq!(
        request.path,
        "/api/v1/technical_geo_reports/7/revert_content"
    );
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({ "project_id": 42, "report_type": "llms_txt" })
    );
    handle.join().unwrap();
}

#[test]
fn sends_the_result_version_with_a_geo_writer_edit() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"id":12,"changed_paths":["title"]}"#);
    let output = run(
        &base_url,
        &[
            "intelligence-tasks",
            "update",
            "12",
            "--edits",
            r#"{"title":"New title"}"#,
            "--result-version",
            "1775642580",
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
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.path, "/api/v1/intelligence_tasks/12");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(body["project_id"], 42);
    assert_eq!(body["result_version"], "1775642580");
    assert_eq!(body["edits"], serde_json::json!({ "title": "New title" }));
    handle.join().unwrap();
}

#[test]
fn leaves_the_result_version_out_of_a_geo_writer_edit_without_one() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"id":12,"changed_paths":["title"]}"#);
    let output = run(
        &base_url,
        &[
            "intelligence-tasks",
            "update",
            "12",
            "--edits",
            r#"{"title":"New title"}"#,
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
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({ "project_id": 42, "edits": { "title": "New title" } })
    );
    handle.join().unwrap();
}

#[test]
fn sends_the_additional_owned_domains_with_a_project_update() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"project":{"id":123}}"#);
    let output = run(
        &base_url,
        &[
            "projects",
            "update",
            "123",
            "--additional-owned-domains",
            "acme-blog.com,acme.co.uk",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let request = receiver.recv().unwrap();
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.path, "/api/v1/projects/123");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({ "additional_owned_domains": ["acme-blog.com", "acme.co.uk"] })
    );
    handle.join().unwrap();
}

#[test]
fn clears_the_additional_owned_domains_with_an_empty_list() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"project":{"id":123}}"#);
    let output = run(
        &base_url,
        &[
            "projects",
            "update",
            "123",
            "--clear-additional-owned-domains",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let request = receiver.recv().unwrap();
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(body, serde_json::json!({ "additional_owned_domains": [] }));
    handle.join().unwrap();
}

#[test]
fn refuses_a_list_and_a_clear_of_the_additional_owned_domains_together() {
    let output = run(
        "http://127.0.0.1:9/api/v1",
        &[
            "projects",
            "update",
            "123",
            "--additional-owned-domains",
            "acme-blog.com",
            "--clear-additional-owned-domains",
        ],
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cannot be used with"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pushes_ai_orders_from_a_json_file_with_put() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"stored":1,"ignored":0}"#);
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("orders.json");
    std::fs::write(
        &file,
        r#"{"platform":"shopify","currency":"EUR","from":"2026-09-01","to":"2026-09-30","days":[{"day":"2026-09-10","referrer":"chatgpt.com","orders":2,"revenue":"40.10"}]}"#,
    )
    .unwrap();
    let output = run(
        &base_url,
        &[
            "store",
            "push-orders",
            "--file",
            file.to_str().unwrap(),
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
    assert_eq!(request.method, "PUT");
    assert_eq!(request.path, "/api/v1/ai_orders");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(body["project_id"], 42);
    assert_eq!(body["days"][0]["referrer"], "chatgpt.com");
    handle.join().unwrap();
}

#[test]
fn store_connection_does_not_require_a_project() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"project":null,"candidates":[]}"#);
    let output = run(
        &base_url,
        &["store", "connection", "--domain", "acme-store.com"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        receiver.recv().unwrap().path,
        "/api/v1/store_connection?platform=shopify&domain=acme-store.com"
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

#[test]
fn creates_geo_audits_with_a_json_body() {
    let (base_url, receiver, handle) = mock_server(201, r#"{"data":[]}"#);
    let output = run(
        &base_url,
        &[
            "geo-audits",
            "create",
            "--target",
            "example.com",
            "--audit-types",
            "robots_txt,schema",
            "--cadence",
            "weekly",
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
    assert_eq!(request.path, "/api/v1/geo_audits");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "project_id": 42,
            "target": "example.com",
            "audit_types": ["robots_txt", "schema"],
            "cadence": "weekly",
        })
    );
    handle.join().unwrap();
}

#[test]
fn updates_a_geo_audit_with_only_the_given_fields() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"id":"abc"}"#);
    let output = run(
        &base_url,
        &[
            "geo-audits",
            "update",
            "abc",
            "--schedule-day",
            "0",
            "--email-alerts",
            "false",
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
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.path, "/api/v1/geo_audits/abc");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({ "project_id": 42, "schedule_day": 0, "email_alerts": false })
    );
    handle.join().unwrap();
}

#[test]
fn refuses_a_geo_audit_update_without_fields() {
    let output = run(
        "http://127.0.0.1:9/api/v1",
        &["geo-audits", "update", "abc", "--project-id", "42"],
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Provide at least one GEO audit field to update")
    );
}

#[test]
fn archiving_a_geo_audit_requires_yes_without_a_terminal() {
    let output = Command::new(binary())
        .args(["geo-audits", "delete", "abc", "--project-id", "42"])
        .args([
            "--api-key",
            "test_key",
            "--base-url",
            "http://127.0.0.1:9/api/v1",
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Archiving GEO audit abc requires --yes")
    );
}

#[test]
fn reopens_a_geo_audit_issue() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"id":9}"#);
    let output = run(
        &base_url,
        &[
            "geo-audits",
            "issues",
            "update",
            "abc",
            "9",
            "--accepted",
            "false",
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
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.path, "/api/v1/geo_audits/abc/issues/9");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({ "project_id": 42, "accepted": false })
    );
    handle.join().unwrap();
}

#[test]
fn reads_the_web_analytics_schema_for_the_project() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"provider":"plausible","rules":[]}"#);
    let output = run(
        &base_url,
        &["web-analytics", "schema", "--project-id", "42"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let request = receiver.recv().unwrap();
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/api/v1/web_analytics/schema?project_id=42");
    handle.join().unwrap();
}

#[test]
fn sends_a_json_web_analytics_query_as_an_object() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"columns":[],"rows":[]}"#);
    let output = run(
        &base_url,
        &[
            "web-analytics",
            "query",
            "--query",
            r#"{"metrics":["visitors"],"date_range":"7d"}"#,
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
    assert_eq!(request.path, "/api/v1/web_analytics/query");
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body,
        serde_json::json!({ "project_id": 42, "query": { "metrics": ["visitors"], "date_range": "7d" } })
    );
    handle.join().unwrap();
}

#[test]
fn sends_hogql_from_a_file_as_a_string() {
    let (base_url, receiver, handle) = mock_server(200, r#"{"columns":[],"rows":[]}"#);
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("query.sql");
    std::fs::write(&file, "SELECT event, count() FROM events GROUP BY event\n").unwrap();
    let output = run(
        &base_url,
        &[
            "web-analytics",
            "query",
            "--file",
            file.to_str().unwrap(),
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
    let body: serde_json::Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(
        body["query"],
        "SELECT event, count() FROM events GROUP BY event"
    );
    handle.join().unwrap();
}
