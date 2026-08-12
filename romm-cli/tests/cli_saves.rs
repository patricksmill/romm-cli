#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;

#[tokio::test]
async fn saves_list_returns_json_array() {
    let server = MockServer::start_async().await;
    let _saves = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/saves")
                .query_param("rom_id", "42");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{"id":9,"file_name":"game.sav","rom_id":42}]"#);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_TOKEN", "test-token")
        .args(["saves", "list", "--rom-id", "42", "--json"]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains(r#""file_name": "game.sav""#));
}

#[tokio::test]
async fn saves_download_writes_file() {
    let server = MockServer::start_async().await;
    let _content = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/saves/9/content");
            then.status(200).body(b"save-bytes");
        })
        .await;

    let out = std::env::temp_dir().join(format!(
        "romm-save-dl-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_TOKEN", "test-token")
        .args(["saves", "download", "9", "--output", out.to_str().unwrap()]);

    cmd.assert().success();
    assert_eq!(std::fs::read(&out).unwrap(), b"save-bytes");
    let _ = std::fs::remove_file(out);
}

#[tokio::test]
async fn saves_upload_posts_multipart() {
    let server = MockServer::start_async().await;
    let upload = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/saves")
                .query_param("rom_id", "42");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"id":1,"file_name":"up.sav"}"#);
        })
        .await;

    let file = std::env::temp_dir().join(format!(
        "romm-save-up-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&file, b"upload-me").unwrap();

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_TOKEN", "test-token")
        .args(["saves", "upload", "--rom-id", "42", file.to_str().unwrap()]);

    cmd.assert().success();
    assert_eq!(upload.hits_async().await, 1);
    let _ = std::fs::remove_file(file);
}
