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
