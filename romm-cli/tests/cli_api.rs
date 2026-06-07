#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;

#[tokio::test]
async fn api_warns_on_malformed_query_pairs() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/platforms")
                .query_param("ok", "1");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"status":"ok"}"#);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("api")
        .arg("GET")
        .arg("/api/platforms")
        .arg("--query")
        .arg("broken")
        .arg("--query")
        .arg("ok=1");

    cmd.assert().success().stderr(predicates::str::contains(
        "warning: ignoring malformed --query value",
    ));
}

#[tokio::test]
async fn api_env_overrides_config_json() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/ping");
            then.status(204);
        })
        .await;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config_dir = std::env::temp_dir().join(format!("romm-cli-test-env-override-{}", ts));
    std::fs::create_dir_all(&config_dir).unwrap();

    let config_json = r#"{
        "base_url": "https://fake.example.com",
        "download_dir": "/tmp/downloads",
        "use_https": true,
        "auth": null
    }"#;
    std::fs::write(config_dir.join("config.json"), config_json).unwrap();

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("api")
        .arg("GET")
        .arg("/api/ping");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("null"));

    let _ = std::fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn api_empty_204_body_prints_null_json() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/ping");
            then.status(204);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("api")
        .arg("GET")
        .arg("/api/ping");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("null"));
}
