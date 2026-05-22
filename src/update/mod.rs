use anyhow::{anyhow, bail, Context, Result};
use self_update::cargo_crate_version;
use self_update::update::ReleaseUpdate;
use self_update::Extract;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env::consts::EXE_SUFFIX;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::io::AsyncWriteExt;

use crate::core::interrupt::{cancelled_error, InterruptContext};

const REPO_OWNER: &str = "patricksmill";
const REPO_NAME: &str = "romm-cli";
const DEFAULT_BIN_NAME: &str = "romm-cli";
const SHIPPED_BINARIES: &[&str] = &["romm-cli", "romm-tui"];
const CHANGELOG_URL: &str = "https://github.com/patricksmill/romm-cli/blob/main/CHANGELOG.md";
const CHECKSUMS_ASSET_NAME: &str = "checksums.txt";

#[derive(Debug, Clone)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: String,
    pub release_tag: String,
    pub should_update: bool,
    pub release_url: String,
    pub changelog_url: String,
}

#[derive(Debug, Clone)]
pub struct ApplyUpdateOptions {
    pub show_progress: bool,
    pub show_output: bool,
    pub no_confirm: bool,
    pub target_version_tag: Option<String>,
}

impl Default for ApplyUpdateOptions {
    fn default() -> Self {
        Self {
            show_progress: false,
            show_output: false,
            no_confirm: true,
            target_version_tag: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyUpdateOutcome {
    Updated(String),
    UpToDate(String),
}

#[derive(Debug, Deserialize)]
struct GithubLatestRelease {
    tag_name: String,
    html_url: String,
}

#[derive(Debug, Clone)]
struct ResolvedRelease {
    version: String,
    archive_name: String,
    archive_download_url: String,
    checksums_download_url: String,
}

pub fn github_api_base_url() -> String {
    std::env::var("ROMM_GITHUB_API_BASE").unwrap_or_else(|_| "https://api.github.com".to_string())
}

fn github_latest_release_api_url() -> String {
    std::env::var("ROMM_GITHUB_LATEST_RELEASE_API").unwrap_or_else(|_| {
        format!(
            "{}/repos/{}/{}/releases/latest",
            github_api_base_url(),
            REPO_OWNER,
            REPO_NAME
        )
    })
}

pub fn github_release_asset_key() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => Ok("macos-x86_64"),
        ("macos", "aarch64") => Ok("macos-aarch64"),
        ("linux", "x86_64") => Ok("linux-x86_64"),
        ("linux", "aarch64") => Ok("linux-aarch64"),
        ("windows", "x86_64") => Ok("windows-x86_64"),
        (os, arch) => Err(anyhow!("unsupported platform for self-update: {os}-{arch}")),
    }
}

fn normalize_version_tag(version: &str) -> &str {
    version.trim().trim_start_matches('v')
}

fn is_latest_newer(latest: &str, current: &str) -> bool {
    self_update::version::bump_is_greater(
        normalize_version_tag(current),
        normalize_version_tag(latest),
    )
    .unwrap_or(false)
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
    let raw = path.as_os_str().to_string_lossy();
    raw.rsplit(['/', '\\'])
        .next()
        .map(|name| {
            name.strip_suffix(".exe")
                .or_else(|| name.strip_suffix(".EXE"))
                .unwrap_or(name)
                .to_string()
        })
        .filter(|name| !name.is_empty())
}

fn current_binary_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| binary_name_from_path(&path))
        .unwrap_or_else(|| DEFAULT_BIN_NAME.to_string())
}

fn shipped_binary_file_name(stem: &str) -> String {
    format!("{stem}{EXE_SUFFIX}")
}

fn build_release_updater(options: &ApplyUpdateOptions) -> Result<Box<dyn ReleaseUpdate>> {
    let target = github_release_asset_key()?;
    let bin_name = current_binary_name();
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(&bin_name)
        .target(target)
        .identifier(DEFAULT_BIN_NAME)
        .current_version(cargo_crate_version!())
        .with_url(&github_api_base_url())
        .show_download_progress(false)
        .show_output(options.show_output)
        .no_confirm(options.no_confirm);

    if let Some(ref tag) = options.target_version_tag {
        builder.target_version_tag(tag);
    }

    builder
        .build()
        .map_err(|e| anyhow!("build self_update config: {e}"))
}

fn resolve_release(options: &ApplyUpdateOptions) -> Result<Option<ResolvedRelease>> {
    let current_version = cargo_crate_version!().to_string();
    let target = github_release_asset_key()?;
    let updater = build_release_updater(options)?;

    let release = if let Some(ref tag) = options.target_version_tag {
        updater.get_release_version(tag)?
    } else {
        let latest = updater.get_latest_release()?;
        if !is_latest_newer(&latest.version, &current_version) {
            return Ok(None);
        }
        latest
    };

    let archive = release
        .asset_for(target, Some(DEFAULT_BIN_NAME))
        .ok_or_else(|| anyhow!("no release asset found for target `{target}`"))?;

    let checksums_download_url = release
        .assets
        .iter()
        .find(|asset| asset.name == CHECKSUMS_ASSET_NAME)
        .ok_or_else(|| anyhow!("release is missing `{CHECKSUMS_ASSET_NAME}` asset"))?
        .download_url
        .clone();

    Ok(Some(ResolvedRelease {
        version: release.version,
        archive_name: archive.name,
        archive_download_url: archive.download_url,
        checksums_download_url,
    }))
}

fn parse_checksums(content: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((hash, name)) = line.split_once("  ") else {
            continue;
        };
        let name = name.trim_start_matches('*').trim();
        out.insert(name.to_string(), hash.to_lowercase());
    }
    out
}

fn sha256_hex_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).context("read file for sha256")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn verify_archive_checksum(
    archive_path: &Path,
    archive_name: &str,
    checksums_content: &str,
) -> Result<()> {
    let checksums = parse_checksums(checksums_content);
    let expected = checksums
        .get(archive_name)
        .ok_or_else(|| anyhow!("checksums.txt has no entry for `{archive_name}`"))?;
    let actual = sha256_hex_file(archive_path)?;
    if &actual != expected {
        bail!("checksum mismatch for `{archive_name}`: expected {expected}, got {actual}");
    }
    Ok(())
}

fn github_asset_headers(user_agent: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_str(user_agent)
            .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("romm-cli")),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/octet-stream"),
    );
    headers
}

async fn download_url_to_file(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    user_agent: &str,
    interrupt: &InterruptContext,
    show_progress: bool,
) -> Result<()> {
    if interrupt.is_cancelled() {
        return Err(cancelled_error());
    }

    let response = client
        .get(url)
        .headers(github_asset_headers(user_agent))
        .send()
        .await
        .with_context(|| format!("download request failed for {url}"))?
        .error_for_status()
        .with_context(|| format!("download returned error status for {url}"))?;

    let total = response.content_length();
    let mut file = tokio::fs::File::create(dest)
        .await
        .with_context(|| format!("create {}", dest.display()))?;

    let progress = if show_progress {
        total.map(|len| {
            let pb = indicatif::ProgressBar::new(len);
            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("{wide_bar} {bytes}/{total_bytes}")
                    .expect("progress template"),
            );
            pb
        })
    } else {
        None
    };

    let mut downloaded = 0u64;
    let mut response = response;
    while let Some(chunk) = response.chunk().await.context("read download chunk")? {
        if interrupt.is_cancelled() {
            return Err(cancelled_error());
        }
        file.write_all(&chunk)
            .await
            .context("write download chunk")?;
        downloaded += chunk.len() as u64;
        if let Some(ref pb) = progress {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    Ok(())
}

fn install_extracted_binaries(extract_dir: &Path, running_bin_stem: &str) -> Result<()> {
    let current_exe = std::env::current_exe().context("resolve current executable path")?;
    let install_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow!("current executable has no parent directory"))?;

    let mut running_source = None;

    for stem in SHIPPED_BINARIES {
        let file_name = shipped_binary_file_name(stem);
        let source = extract_dir.join(&file_name);
        if !source.is_file() {
            continue;
        }

        let dest = install_dir.join(&file_name);
        if stem == &running_bin_stem {
            running_source = Some(source);
            continue;
        }

        std::fs::copy(&source, &dest).with_context(|| {
            format!(
                "copy sibling binary `{}` to `{}`",
                source.display(),
                dest.display()
            )
        })?;
        if let Ok(meta) = std::fs::metadata(&source) {
            let _ = std::fs::set_permissions(&dest, meta.permissions());
        }
    }

    let Some(new_running) = running_source else {
        bail!("extracted archive did not contain `{running_bin_stem}`");
    };

    self_update::self_replace::self_replace(new_running).context("replace running executable")?;

    Ok(())
}

fn install_from_archive(
    archive_path: &Path,
    archive_name: &str,
    checksums_content: &str,
) -> Result<()> {
    verify_archive_checksum(archive_path, archive_name, checksums_content)?;

    let extract_dir = self_update::TempDir::new().context("create temp extract dir")?;
    Extract::from_source(archive_path)
        .extract_into(extract_dir.path())
        .with_context(|| format!("extract `{archive_name}`"))?;

    install_extracted_binaries(extract_dir.path(), &current_binary_name())?;
    Ok(())
}

pub async fn check_for_update() -> Result<UpdateStatus> {
    let current_version = cargo_crate_version!().to_string();
    let response = reqwest::Client::new()
        .get(github_latest_release_api_url())
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

    let release_tag = latest_release.tag_name.clone();
    let latest_version = release_tag.trim_start_matches('v').to_string();
    Ok(UpdateStatus {
        should_update: is_latest_newer(&latest_version, &current_version),
        current_version,
        latest_version,
        release_tag,
        release_url: latest_release.html_url,
        changelog_url: changelog_url().to_string(),
    })
}

pub async fn apply_update(
    interrupt: Option<InterruptContext>,
    options: ApplyUpdateOptions,
) -> Result<ApplyUpdateOutcome> {
    let interrupt = interrupt.unwrap_or_default();
    let current_version = cargo_crate_version!().to_string();
    let user_agent = format!("romm-cli/{current_version}");

    let resolved = tokio::task::spawn_blocking({
        let options = options.clone();
        move || resolve_release(&options)
    })
    .await
    .map_err(|e| anyhow!("update resolve task failed: {e}"))??;

    let Some(resolved) = resolved else {
        return Ok(ApplyUpdateOutcome::UpToDate(current_version));
    };

    let archive_dir = self_update::TempDir::new().context("create temp download dir")?;
    let archive_path: PathBuf = archive_dir.path().join(&resolved.archive_name);

    let client = reqwest::Client::new();

    if interrupt.is_cancelled() {
        return Err(cancelled_error());
    }
    let checksums_content = client
        .get(&resolved.checksums_download_url)
        .headers(github_asset_headers(&user_agent))
        .send()
        .await
        .context("download checksums.txt")?
        .error_for_status()
        .context("checksums.txt request failed")?
        .text()
        .await
        .context("read checksums.txt")?;

    download_url_to_file(
        &client,
        &resolved.archive_download_url,
        &archive_path,
        &user_agent,
        &interrupt,
        options.show_progress,
    )
    .await?;

    let version = resolved.version.clone();
    let archive_name = resolved.archive_name.clone();
    let install_task = tokio::task::spawn_blocking(move || {
        install_from_archive(&archive_path, &archive_name, &checksums_content).map(|_| version)
    });

    let installed_version = tokio::select! {
        out = install_task => out
            .map_err(|e| anyhow!("update install task failed: {e}"))??,
        _ = interrupt.cancelled() => return Err(cancelled_error()),
    };

    Ok(ApplyUpdateOutcome::Updated(installed_version))
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
    fn version_compare_handles_prerelease_to_stable() {
        assert!(is_latest_newer("0.25.0", "0.25.0-alpha"));
    }

    #[test]
    fn parse_checksums_reads_sha256sum_format() {
        let parsed = parse_checksums("abc123  romm-cli-linux-x86_64.tar.gz\n");
        assert_eq!(
            parsed.get("romm-cli-linux-x86_64.tar.gz"),
            Some(&"abc123".to_string())
        );
    }

    #[test]
    fn verify_archive_checksum_matches() {
        let dir = self_update::TempDir::new().expect("tempdir");
        let path = dir.path().join("sample.tar.gz");
        std::fs::write(&path, b"hello").expect("write sample");
        let digest = sha256_hex_file(&path).expect("hash");
        let checksums = format!("{digest}  sample.tar.gz\n");
        verify_archive_checksum(&path, "sample.tar.gz", &checksums).expect("verify");
    }

    #[test]
    fn verify_archive_checksum_rejects_mismatch() {
        let dir = self_update::TempDir::new().expect("tempdir");
        let path = dir.path().join("sample.tar.gz");
        std::fs::write(&path, b"hello").expect("write sample");
        let checksums = "deadbeef  sample.tar.gz\n";
        assert!(verify_archive_checksum(&path, "sample.tar.gz", checksums).is_err());
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

    #[test]
    fn github_release_asset_key_supports_windows() {
        if std::env::consts::OS == "windows" && std::env::consts::ARCH == "x86_64" {
            assert_eq!(
                github_release_asset_key().expect("target"),
                "windows-x86_64"
            );
        }
    }
}
