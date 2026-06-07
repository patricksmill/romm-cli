//! Frontend-aware wrappers around [`romm_api::update`] for the TUI binary.

pub use romm_api::update::{
    ApplyUpdateOptions, ApplyUpdateOutcome, ReleaseComponent, UpdateContext, UpdateStatus,
};

/// Check for updates using the `romm-tui` crate version and running binary name.
pub async fn check_for_update() -> anyhow::Result<UpdateStatus> {
    romm_api::update::check_for_update(UpdateContext::for_running_binary(env!("CARGO_PKG_VERSION")))
        .await
}

/// Apply a self-update using the `romm-tui` crate version and running binary name.
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
    ReleaseComponent::RommTui.changelog_url()
}

pub fn open_changelog_in_browser() -> anyhow::Result<()> {
    romm_api::update::open_changelog_in_browser(ReleaseComponent::RommTui)
}
