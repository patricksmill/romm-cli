#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;
use romm_cli::update;

#[test]
fn update_subcommand_is_registered() {
    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.arg("update").arg("--help");
    cmd.assert().success();
}

#[tokio::test]
async fn check_for_update_reads_mocked_github_latest() {
    let server = MockServer::start_async().await;

    let _mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/repos/patricksmill/romm-cli/releases/latest");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "tag_name": "v999.0.0",
                        "html_url": "https://github.com/patricksmill/romm-cli/releases/tag/v999.0.0"
                    }"#,
                );
        })
        .await;

    let api_url = format!(
        "{}/repos/patricksmill/romm-cli/releases/latest",
        server.base_url()
    );
    std::env::set_var("ROMM_GITHUB_LATEST_RELEASE_API", &api_url);

    let status = update::check_for_update().await.expect("check_for_update");
    assert!(status.should_update);
    assert_eq!(status.latest_version, "999.0.0");
    assert_eq!(status.release_tag, "v999.0.0");

    std::env::remove_var("ROMM_GITHUB_LATEST_RELEASE_API");
}

#[test]
fn github_api_base_url_respects_env_override() {
    let previous = std::env::var("ROMM_GITHUB_API_BASE").ok();

    std::env::remove_var("ROMM_GITHUB_API_BASE");
    assert_eq!(update::github_api_base_url(), "https://api.github.com");

    std::env::set_var("ROMM_GITHUB_API_BASE", "http://127.0.0.1:9");
    assert_eq!(update::github_api_base_url(), "http://127.0.0.1:9");

    match previous {
        Some(value) => std::env::set_var("ROMM_GITHUB_API_BASE", value),
        None => std::env::remove_var("ROMM_GITHUB_API_BASE"),
    }
}
