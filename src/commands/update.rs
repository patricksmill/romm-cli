use anyhow::Result;
use self_update::cargo_crate_version;

use crate::core::interrupt::{cancelled_error, InterruptContext};

/// Substring inside each GitHub release archive name (`romm-cli-….tar.gz` / `.zip`).
/// `self_update` matches on this; our assets use `macos-x86_64` etc., not the full Rust triple.
fn github_release_asset_key() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("windows", "x86_64") => "windows-x86_64",
        _ => self_update::get_target(),
    }
}

/// Handle the `update` command.
pub async fn handle(interrupt: Option<InterruptContext>) -> Result<()> {
    let interrupt = interrupt.unwrap_or_default();
    let update_task = tokio::task::spawn_blocking(|| -> Result<String> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner("patricksmill")
            .repo_name("romm-cli")
            .bin_name("romm-cli")
            .target(github_release_asset_key())
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;
        Ok(status.version().to_string())
    });

    let version = tokio::select! {
        out = update_task => out
            .map_err(|e| anyhow::anyhow!("update task failed: {e}"))??,
        _ = interrupt.cancelled() => return Err(cancelled_error()),
    };

    println!("Update status: `{}`!", version);
    Ok(())
}
