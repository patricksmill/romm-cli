#![allow(deprecated)]

use assert_cmd::Command;
use romm_api::update::{self, ReleaseComponent, UpdateContext};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn update_subcommand_is_registered() {
    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.arg("update").arg("--help");
    cmd.assert().success();
}

#[tokio::test]
async fn check_for_update_reads_mocked_component_releases() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/patricksmill/romm-cli/releases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "tag_name": "romm-cli-v999.0.0",
                "html_url": "https://github.com/patricksmill/romm-cli/releases/tag/romm-cli-v999.0.0"
            }
        ])))
        .mount(&server)
        .await;

    std::env::set_var(
        "ROMM_GITHUB_RELEASES_API",
        format!("{}/repos/patricksmill/romm-cli/releases", server.uri()),
    );

    let ctx = UpdateContext::for_running_binary(env!("CARGO_PKG_VERSION"));
    let status = update::check_for_update(ctx)
        .await
        .expect("check_for_update");
    assert!(status.should_update);
    assert_eq!(status.latest_version, "999.0.0");
    assert_eq!(status.release_tag, "romm-cli-v999.0.0");

    std::env::remove_var("ROMM_GITHUB_RELEASES_API");
}

#[test]
fn select_latest_release_tag_cli_component() {
    let tags = ["romm-cli-v0.40.0", "romm-cli-v999.0.0", "romm-tui-v1.0.0"];
    assert_eq!(
        update::select_latest_release_tag(ReleaseComponent::RommCli, tags.iter().copied()),
        Some("romm-cli-v999.0.0".to_string())
    );
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
