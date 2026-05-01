use anyhow::{anyhow, Context, Result};
use self_update::cargo_crate_version;
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use crate::core::interrupt::{cancelled_error, InterruptContext};

const REPO_OWNER: &str = "patricksmill";
const REPO_NAME: &str = "romm-cli";
const DEFAULT_BIN_NAME: &str = "romm-cli";
const GITHUB_LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/patricksmill/romm-cli/releases/latest";
const CHANGELOG_URL: &str = "https://github.com/patricksmill/romm-cli/blob/main/CHANGELOG.md";

#[derive(Debug, Clone)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: String,
    pub should_update: bool,
    pub release_url: String,
    pub changelog_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubLatestRelease {
    tag_name: String,
    html_url: String,
}

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

fn parse_numeric_version_parts(input: &str) -> Vec<u64> {
    let trimmed = input.trim().trim_start_matches('v');
    trimmed
        .split(['.', '-'])
        .take(3)
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

fn is_latest_newer(latest: &str, current: &str) -> bool {
    let mut latest_parts = parse_numeric_version_parts(latest);
    let mut current_parts = parse_numeric_version_parts(current);
    let max_len = latest_parts.len().max(current_parts.len()).max(3);
    latest_parts.resize(max_len, 0);
    current_parts.resize(max_len, 0);
    latest_parts > current_parts
}

pub fn changelog_url() -> &'static str {
    CHANGELOG_URL
}

pub fn open_url_in_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .context("failed to launch browser via start")?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .context("failed to launch browser via open")?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("failed to launch browser via xdg-open")?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(anyhow!("unsupported OS for opening browser"))
}

pub fn open_changelog_in_browser() -> Result<()> {
    open_url_in_browser(changelog_url())
}

fn binary_name_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
}

fn current_binary_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| binary_name_from_path(&path))
        .unwrap_or_else(|| DEFAULT_BIN_NAME.to_string())
}

pub async fn check_for_update() -> Result<UpdateStatus> {
    let current_version = cargo_crate_version!().to_string();
    let response = reqwest::Client::new()
        .get(GITHUB_LATEST_RELEASE_API)
        .header(
            reqwest::header::USER_AGENT,
            format!("romm-cli/{current_version}"),
        )
        .send()
        .await
        .context("failed to query latest release")?
        .error_for_status()
        .context("latest release endpoint returned an error status")?;

    let latest_release: GithubLatestRelease = response
        .json()
        .await
        .context("failed to parse latest release response")?;

    let latest_version = latest_release.tag_name.trim_start_matches('v').to_string();
    Ok(UpdateStatus {
        should_update: is_latest_newer(&latest_version, &current_version),
        current_version,
        latest_version,
        release_url: latest_release.html_url,
        changelog_url: changelog_url().to_string(),
    })
}

pub async fn apply_update(interrupt: Option<InterruptContext>) -> Result<String> {
    let interrupt = interrupt.unwrap_or_default();
    let bin_name = current_binary_name();
    let update_task = tokio::task::spawn_blocking(move || -> Result<String> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .bin_name(&bin_name)
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
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare_handles_patch_and_minor() {
        assert!(is_latest_newer("0.25.1", "0.25.0"));
        assert!(is_latest_newer("0.26.0", "0.25.9"));
        assert!(!is_latest_newer("0.25.0", "0.25.0"));
        assert!(!is_latest_newer("0.24.9", "0.25.0"));
    }

    #[test]
    fn version_compare_handles_v_prefix() {
        assert!(is_latest_newer("v1.2.4", "1.2.3"));
    }

    #[test]
    fn binary_name_from_path_strips_windows_exe_extension() {
        assert_eq!(
            binary_name_from_path(Path::new(r"C:\tools\romm-tui.exe")).as_deref(),
            Some("romm-tui")
        );
    }

    #[test]
    fn current_binary_name_is_available() {
        assert!(!current_binary_name().is_empty());
    }
}
