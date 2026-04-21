//! Integration tests for `romm-cli roms` command.
//! Verifies that platform_ids (RomM API param name) is sent so the API filters by console.

#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;

#[tokio::test]
async fn roms_sends_platform_ids_query_param() {
    let server = MockServer::start_async().await;

    let platforms_body = r#"[{
        "id": 5,
        "slug": "play5",
        "fs_slug": "play5",
        "rom_count": 0,
        "name": "Play Five",
        "igdb_slug": null,
        "moby_slug": null,
        "hltb_slug": null,
        "custom_name": null,
        "igdb_id": null,
        "sgdb_id": null,
        "moby_id": null,
        "launchbox_id": null,
        "ss_id": null,
        "ra_id": null,
        "hasheous_id": null,
        "tgdb_id": null,
        "flashpoint_id": null,
        "category": null,
        "generation": null,
        "family_name": null,
        "family_slug": null,
        "url": null,
        "url_logo": null,
        "firmware": [],
        "aspect_ratio": null,
        "created_at": "",
        "updated_at": "",
        "fs_size_bytes": 0,
        "is_unidentified": false,
        "is_identified": true,
        "missing_from_fs": false,
        "display_name": "Play Five"
    }]"#;

    let rom_list_body = r#"{"items":[],"total":0,"limit":500,"offset":0}"#;

    let _platforms_mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/platforms");
            then.status(200)
                .header("content-type", "application/json")
                .body(platforms_body);
        })
        .await;

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
        .env("API_USE_HTTPS", "false")
        .env("API_USERNAME", "u")
        .env("API_PASSWORD", "p")
        .args(["roms", "list", "--platform", "play5"]);

    cmd.assert().success();
    mock.assert();
}

#[tokio::test]
async fn roms_sends_search_term_query_param() {
    let server = MockServer::start_async().await;

    let rom_list_body = r#"{"items":[],"total":0,"limit":50,"offset":0}"#;

    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms")
                .query_param("search_term", "zelda");
            then.status(200)
                .header("content-type", "application/json")
                .body(rom_list_body);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_USERNAME", "u")
        .env("API_PASSWORD", "p")
        .args(["roms", "list", "--search-term", "zelda"]);

    cmd.assert().success();
    mock.assert();
}
