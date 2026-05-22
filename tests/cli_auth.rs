#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::POST;
use httpmock::MockServer;
use std::fs;

fn test_config_dir(prefix: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("romm-cli-{prefix}-test-{ts}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_minimal_config(dir: &std::path::Path) {
    let config_json = r#"{
        "base_url": "https://disk.example",
        "download_dir": "/tmp/downloads",
        "use_https": true,
        "auth": null
    }"#;
    fs::write(dir.join("config.json"), config_json).unwrap();
}

#[tokio::test]
async fn auth_status_reports_effective_bearer_from_env() {
    let config_dir = test_config_dir("auth-status");
    write_minimal_config(&config_dir);

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", "http://example.test")
        .env("API_USE_HTTPS", "false")
        .env("API_TOKEN", "test-token")
        .args(["--json", "auth", "status"]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains(r#""mode": "bearer""#));

    let _ = fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn auth_login_persists_bearer_token_to_disk() {
    let config_dir = test_config_dir("auth-login");
    write_minimal_config(&config_dir);

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", "http://example.test")
        .env("API_USE_HTTPS", "false")
        .args(["auth", "login", "--token", "saved-token"]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Auth updated: bearer"));

    let saved = fs::read_to_string(config_dir.join("config.json")).unwrap();
    assert!(saved.contains("saved-token"));

    let _ = fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn auth_logout_clears_disk_auth() {
    let config_dir = test_config_dir("auth-logout");
    let config_json = r#"{
        "base_url": "https://disk.example",
        "download_dir": "/tmp/downloads",
        "use_https": true,
        "auth": { "Bearer": { "token": "old-token" } }
    }"#;
    fs::write(config_dir.join("config.json"), config_json).unwrap();

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", "http://example.test")
        .env("API_USE_HTTPS", "false")
        .args(["auth", "logout"]);

    cmd.assert().success();

    let saved = fs::read_to_string(config_dir.join("config.json")).unwrap();
    assert!(!saved.contains("old-token"));
    assert!(saved.contains(r#""auth": null"#) || saved.contains(r#""auth":null"#));

    let _ = fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn auth_login_exchanges_pairing_code() {
    let server = MockServer::start_async().await;
    let config_dir = test_config_dir("auth-pairing");

    let config_json = format!(
        r#"{{
        "base_url": "{}",
        "download_dir": "/tmp/downloads",
        "use_https": false,
        "auth": null
    }}"#,
        server.base_url()
    );
    fs::write(config_dir.join("config.json"), config_json).unwrap();

    let _mock = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/client-tokens/exchange");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "id": 1,
                        "name": "cli",
                        "scopes": [],
                        "expires_at": null,
                        "last_used_at": null,
                        "created_at": "2020-01-01T00:00:00Z",
                        "user_id": 1,
                        "raw_token": "paired-token"
                    }"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .args(["auth", "login", "--pairing-code", "ABCD1234"]);

    cmd.assert().success();

    let saved = fs::read_to_string(config_dir.join("config.json")).unwrap();
    assert!(saved.contains("paired-token"));

    let _ = fs::remove_dir_all(config_dir);
}
