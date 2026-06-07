#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;
use std::fs;

fn isolated_config_dir(prefix: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("romm-cli-exit-{prefix}-{ts}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn exit_3_missing_config() {
    let config_dir = isolated_config_dir("missing-config");

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env_remove("API_BASE_URL")
        .env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .arg("platforms");

    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicates::str::contains("romm-cli init"));

    let _ = fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn exit_3_unauthorized() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/platforms");
            then.status(401)
                .header("content-type", "application/json")
                .body(r#"{"detail":"bad token"}"#);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("platforms");

    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicates::str::contains("Authentication failed"));
}

#[tokio::test]
async fn exit_4_server_error() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/platforms");
            then.status(503)
                .header("content-type", "application/json")
                .body(r#"{"detail":"unavailable"}"#);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("platforms");

    cmd.assert()
        .failure()
        .code(4)
        .stderr(predicates::str::contains("Server error"));
}

#[test]
fn exit_2_clap_usage() {
    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.arg("--not-a-flag");

    cmd.assert().failure().code(2);
}

#[test]
fn exit_1_init_validation() {
    let config_dir = isolated_config_dir("init-validation");

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .args(["init", "--url", "https://romm.example.com"]);

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "--url requires either --token or --token-file",
        ));

    let _ = fs::remove_dir_all(config_dir);
}
