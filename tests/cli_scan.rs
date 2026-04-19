#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;

#[tokio::test]
async fn scan_triggers_post_run_scan_library() {
    let server = MockServer::start_async().await;

    let _run = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/tasks/run/scan_library");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"task_name":"Scheduled rescan","task_id":"job-abc","status":"queued","created_at":"2020-01-01T00:00:00Z","enqueued_at":"2020-01-01T00:00:00Z"}"#,
                );
        })
        .await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config_dir = std::env::temp_dir().join(format!("romm-cli-scan-test-{}", ts));
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_json = r#"{
        "base_url": "https://unused.example.com",
        "download_dir": "/tmp/downloads",
        "use_https": true,
        "auth": null
    }"#;
    std::fs::write(config_dir.join("config.json"), config_json).unwrap();

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("scan");

    cmd.assert().success().stdout(predicates::str::contains("job-abc"));

    let _ = std::fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn scan_wait_polls_until_finished() {
    let server = MockServer::start_async().await;

    let _run = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/tasks/run/scan_library");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"task_name":"Scheduled rescan","task_id":"job-wait","status":"queued","created_at":"2020-01-01T00:00:00Z","enqueued_at":"2020-01-01T00:00:00Z"}"#,
                );
        })
        .await;

    let _poll = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/tasks/job-wait");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"task_id":"job-wait","status":"finished","task_name":"scan"}"#);
        })
        .await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config_dir = std::env::temp_dir().join(format!("romm-cli-scan-wait-test-{}", ts));
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_json = r#"{
        "base_url": "https://unused.example.com",
        "download_dir": "/tmp/downloads",
        "use_https": true,
        "auth": null
    }"#;
    std::fs::write(config_dir.join("config.json"), config_json).unwrap();

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("scan")
        .arg("--wait")
        .arg("--wait-timeout-secs")
        .arg("30");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("finished successfully"));

    let _ = std::fs::remove_dir_all(config_dir);
}
