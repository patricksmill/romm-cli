#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;

fn has_ansi(s: &str) -> bool {
    s.contains("\x1b[")
}

fn isolated_config_dir(prefix: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("romm-cli-output-{prefix}-{ts}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn json_stdout_has_no_ansi_with_no_color() {
    let server = MockServer::start_async().await;

    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/platforms");
            then.status(200)
                .header("content-type", "application/json")
                .body("[]");
        })
        .await;

    let config_dir = isolated_config_dir("no-color-json");

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("NO_COLOR", "1")
        .env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args(["--json", "platforms", "list"]);

    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(!has_ansi(&stdout), "stdout contained ANSI: {stdout}");
    serde_json::from_str::<serde_json::Value>(&stdout).expect("stdout must be valid JSON");

    let _ = std::fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn scan_json_wait_stdout_is_valid_json_only() {
    let server = MockServer::start_async().await;

    let _run = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/tasks/run/scan_library");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"task_name":"scan","task_id":"job-json","status":"queued","created_at":"2020-01-01T00:00:00Z","enqueued_at":"2020-01-01T00:00:00Z"}"#,
                );
        })
        .await;

    let _poll = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/tasks/job-json");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"task_id":"job-json","status":"finished"}"#);
        })
        .await;

    let config_dir = isolated_config_dir("scan-json-wait");

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args(["--json", "scan", "--wait"]);

    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(!has_ansi(&stdout));
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("scan --json --wait stdout must be JSON");
    assert!(value.get("task_id").is_some());
    assert!(value.get("final_status").is_some());

    let _ = std::fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn not_found_error_suggests_check_url() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/platforms");
            then.status(404)
                .header("content-type", "application/json")
                .body(r#"{"detail":"missing"}"#);
        })
        .await;

    let config_dir = isolated_config_dir("not-found-hint");

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("platforms");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("server URL"));

    let _ = std::fs::remove_dir_all(config_dir);
}
