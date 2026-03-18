#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;

#[tokio::test]
async fn platforms_list_text_output() {
    let server = MockServer::start_async().await;

    let platforms_body = r#"[{
        "id": 1,
        "slug": "nes",
        "fs_slug": "nes",
        "rom_count": 42,
        "name": "Nintendo Entertainment System",
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
        "display_name": "NES"
    }]"#;

    let _m = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/platforms");
            then.status(200)
                .header("content-type", "application/json")
                .body(platforms_body);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url()).arg("platforms");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("NES"));
}
