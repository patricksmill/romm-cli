//! Integration tests for `romm-cli roms` command.
//! Verifies that platform_ids (RomM API param name) is sent so the API filters by console.

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;

#[tokio::test]
async fn roms_sends_platform_ids_query_param() {
    let server = MockServer::start_async().await;

    let rom_list_body = r#"{"items":[],"total":0,"limit":500,"offset":0}"#;

    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms")
                .query_param("platform_ids", "5");
            then.status(200)
                .header("content-type", "application/json")
                .body(rom_list_body);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USERNAME", "u")
        .env("API_PASSWORD", "p")
        .arg("roms")
        .arg("--platform-id")
        .arg("5");

    cmd.assert().success();
    mock.assert();
}
