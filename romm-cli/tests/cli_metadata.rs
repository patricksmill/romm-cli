//! Integration tests for `romm-cli roms metadata` commands.

use assert_cmd::Command;
use httpmock::Method::{GET, PUT};
use httpmock::MockServer;

#[tokio::test]
async fn metadata_search_calls_search_roms() {
    let server = MockServer::start_async().await;

    let body = r#"[{
        "name": "Mario",
        "platform_id": 1,
        "igdb_id": 99,
        "is_identified": true,
        "is_unidentified": false
    }]"#;

    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/search/roms")
                .query_param("rom_id", "1")
                .query_param("search_term", "mario");
            then.status(200)
                .header("content-type", "application/json")
                .body(body);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_USERNAME", "u")
        .env("API_PASSWORD", "p")
        .args([
            "roms", "metadata", "search", "1", "--query", "mario", "--json",
        ]);

    cmd.assert().success();
    mock.assert();
}

#[tokio::test]
async fn metadata_match_sends_multipart_put() {
    let server = MockServer::start_async().await;

    let mock = server
        .mock_async(|when, then| {
            when.method(PUT).path("/api/roms/5");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"id":5,"platform_id":2,"name":"Zelda","is_identified":true,"is_unidentified":false}"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_USERNAME", "u")
        .env("API_PASSWORD", "p")
        .args([
            "roms",
            "metadata",
            "match",
            "5",
            "--igdb-id",
            "1234",
            "--json",
        ]);

    cmd.assert().success();
    mock.assert();
}

#[tokio::test]
async fn metadata_unmatch_sends_query_flag() {
    let server = MockServer::start_async().await;

    let mock = server
        .mock_async(|when, then| {
            when.method(PUT)
                .path("/api/roms/3")
                .query_param("unmatch_metadata", "true");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"id":3,"platform_id":1,"name":"Game","is_identified":false,"is_unidentified":true}"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_USERNAME", "u")
        .env("API_PASSWORD", "p")
        .args(["roms", "metadata", "unmatch", "3", "--yes", "--json"]);

    cmd.assert().success();
    mock.assert();
}
