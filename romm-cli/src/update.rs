//! Frontend-aware wrappers around [`romm_api::update`] for the CLI binary.

pub use romm_api::update::{
    open_url_in_browser, ApplyUpdateOptions, ApplyUpdateOutcome, ReleaseComponent, UpdateContext,
    UpdateStatus,
};

/// Check for updates using the `romm-cli` crate version and running binary name.
pub async fn check_for_update() -> anyhow::Result<UpdateStatus> {
    romm_api::update::check_for_update(UpdateContext::for_running_binary(env!("CARGO_PKG_VERSION")))
        .await
}

/// Apply a self-update using the `romm-cli` crate version and running binary name.
pub async fn apply_update(
    interrupt: Option<romm_api::core::interrupt::InterruptContext>,
    options: ApplyUpdateOptions,
) -> anyhow::Result<ApplyUpdateOutcome> {
    romm_api::update::apply_update(
        interrupt,
        options,
        UpdateContext::for_running_binary(env!("CARGO_PKG_VERSION")),
    )
    .await
}

pub fn changelog_url() -> &'static str {
    ReleaseComponent::RommCli.changelog_url()
}

pub fn open_changelog_in_browser() -> anyhow::Result<()> {
    romm_api::update::open_changelog_in_browser(ReleaseComponent::RommCli)
}

pub fn github_api_base_url() -> String {
    romm_api::update::github_api_base_url()
}

pub fn github_release_asset_key() -> anyhow::Result<&'static str> {
    romm_api::update::github_release_asset_key()
}
