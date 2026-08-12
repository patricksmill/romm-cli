#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;

#[tokio::test]
async fn collections_list_all_merges_sources() {
    let server = MockServer::start_async().await;
    let _manual = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/collections");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{"id":1,"name":"Favorites","rom_count":2}]"#);
        })
        .await;
    let _smart = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/collections/smart");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{"id":2,"name":"Recent Smart","rom_count":1}]"#);
        })
        .await;
    let _virtual = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/collections/virtual");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{"id":"recent","name":"Recent Virtual","type":"recent","rom_count":3,"is_virtual":true}]"#);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_TOKEN", "test-token")
        .args(["collections", "list", "--type", "all", "--json"]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Favorites"))
        .stdout(predicates::str::contains("Recent Smart"))
        .stdout(predicates::str::contains("Recent Virtual"));
}
