//! Download state and management.
//!
//! `DownloadJob` holds per-download progress/status.
//! `DownloadManager` owns the shared job list and spawns background tasks.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use crate::client::RommClient;
use crate::core::utils;
use crate::types::Rom;

/// Directory for ROM storage (`ROMM_ROMS_DIR`, `ROMM_DOWNLOAD_DIR`, or configured path).
pub fn resolve_download_directory(configured_download_dir: Option<&str>) -> Result<PathBuf> {
    let env_override = std::env::var("ROMM_ROMS_DIR")
        .ok()
        .or_else(|| std::env::var("ROMM_DOWNLOAD_DIR").ok());
    resolve_download_directory_from_inputs(configured_download_dir, env_override.as_deref())
}

/// Validate configured download path without env override fallback.
pub fn validate_configured_download_directory(configured_download_dir: &str) -> Result<PathBuf> {
    resolve_download_directory_from_inputs(Some(configured_download_dir), None)
}

/// Backward-compatible default used by legacy CLI download code.
pub fn download_directory() -> PathBuf {
    std::env::var("ROMM_ROMS_DIR")
        .or_else(|_| std::env::var("ROMM_DOWNLOAD_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./downloads"))
}

fn resolve_download_directory_from_inputs(
    configured_download_dir: Option<&str>,
    env_override: Option<&str>,
) -> Result<PathBuf> {
    let raw = env_override.or(configured_download_dir).map(str::trim).ok_or_else(|| {
        anyhow!("ROMs directory is not configured. Run setup to set a ROMs path.")
    })?;

    if raw.is_empty() {
        return Err(anyhow!("ROMs directory cannot be empty"));
    }

    let input_path = PathBuf::from(raw);
    let normalized = if input_path.is_relative() {
        std::env::current_dir()
            .context("Could not resolve current working directory")?
            .join(input_path)
    } else {
        input_path
    };

    if normalized.exists() && !normalized.is_dir() {
        return Err(anyhow!(
            "Download path is not a directory: {}",
            normalized.display()
        ));
    }

    std::fs::create_dir_all(&normalized)
        .with_context(|| format!("Could not create download directory {}", normalized.display()))?;

    let probe_name = format!(
        ".romm-write-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let probe_path = normalized.join(probe_name);
    let probe = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)
        .with_context(|| {
            format!(
                "ROMs directory is not writable: {}",
                normalized.display()
            )
        })?;
    drop(probe);
    let _ = std::fs::remove_file(&probe_path);

    Ok(normalized)
}

/// Pick `stem.zip`, then `stem__2.zip`, `stem__3.zip`, … until the path does not exist.
pub fn unique_zip_path(dir: &Path, stem: &str) -> PathBuf {
    let mut n = 1u32;
    loop {
        let name = if n == 1 {
            format!("{}.zip", stem)
        } else {
            format!("{}__{}.zip", stem, n)
        };
        let p = dir.join(name);
        if !p.exists() {
            return p;
        }
        n = n.saturating_add(1);
    }
}

// ---------------------------------------------------------------------------
// Job status / data
// ---------------------------------------------------------------------------

/// High-level status of a single download.
#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Downloading,
    Done,
    SkippedAlreadyExists,
    FinalizeFailed(String),
    Error(String),
}

/// A single background download job (for one ROM).
#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub id: usize,
    pub rom_id: u64,
    pub name: String,
    pub platform: String,
    /// 0.0 ..= 1.0
    pub progress: f64,
    pub status: DownloadStatus,
}

static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(0);

impl DownloadJob {
    /// Construct a new job in the `Downloading` state.
    pub fn new(rom_id: u64, name: String, platform: String) -> Self {
        Self {
            id: NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed),
            rom_id,
            name,
            platform,
            progress: 0.0,
            status: DownloadStatus::Downloading,
        }
    }

    /// Progress as percentage 0..=100.
    pub fn percent(&self) -> u16 {
        (self.progress * 100.0).round().min(100.0) as u16
    }
}

// ---------------------------------------------------------------------------
// Manager
// ---------------------------------------------------------------------------

/// Owns the shared download list and spawns background download tasks.
///
/// Frontends only need an `Arc<Mutex<Vec<DownloadJob>>>` to inspect jobs.
#[derive(Clone)]
pub struct DownloadManager {
    jobs: Arc<Mutex<Vec<DownloadJob>>>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Shared handle for observers (TUI, GUI, tests) to inspect jobs.
    pub fn shared(&self) -> Arc<Mutex<Vec<DownloadJob>>> {
        self.jobs.clone()
    }

    /// Start downloading `rom` in the background; returns immediately.
    ///
    /// Progress updates are pushed into the shared `jobs` list so that
    /// any frontend can render them.
    pub fn start_download(
        &self,
        rom: &Rom,
        client: RommClient,
        configured_download_dir: Option<&str>,
    ) -> Result<()> {
        let platform = rom
            .platform_display_name
            .as_deref()
            .or(rom.platform_custom_name.as_deref())
            .unwrap_or("—")
            .to_string();

        let job = DownloadJob::new(rom.id, rom.name.clone(), platform);
        let job_id = job.id;
        let rom_id = rom.id;
        let fs_name = rom.fs_name.clone();
        let final_console_slug = rom
            .platform_fs_slug
            .clone()
            .or_else(|| rom.platform_slug.clone())
            .unwrap_or_else(|| format!("platform-{}", rom.platform_id));
        let final_name = sanitized_final_filename(&rom.fs_name, rom.id);
        match self.jobs.lock() {
            Ok(mut jobs) => jobs.push(job),
            Err(err) => {
                eprintln!("warning: download job list lock poisoned: {}", err);
                return Err(anyhow!("download job list lock poisoned: {err}"));
            }
        }

        let save_dir = resolve_download_directory(configured_download_dir)?;
        let jobs = self.jobs.clone();
        tokio::spawn(async move {
            let temp_root = save_dir.join(".tmp");
            if let Err(err) = tokio::fs::create_dir_all(&temp_root).await {
                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.status = DownloadStatus::Error(format!(
                            "Could not create temp directory {}: {err}",
                            temp_root.display()
                        ));
                    }
                }
                return;
            }

            let console_dir = save_dir.join(utils::sanitize_filename(&final_console_slug));
            let final_path = console_dir.join(final_name.clone());
            if let Err(err) = tokio::fs::create_dir_all(&console_dir).await {
                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.status = DownloadStatus::Error(format!(
                            "Could not create console directory {}: {err}",
                            console_dir.display()
                        ));
                    }
                }
                return;
            }

            if final_path.exists() {
                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.status = DownloadStatus::SkippedAlreadyExists;
                        j.progress = 1.0;
                    }
                }
                return;
            }

            let temp_name = format!(
                "rom-{}-{}-{}.part",
                rom_id,
                utils::sanitize_filename(&fs_name),
                job_id
            );
            let temp_path = temp_root.join(temp_name);

            let on_progress = |received: u64, total: u64| {
                let p = if total > 0 {
                    received as f64 / total as f64
                } else {
                    0.0
                };

                if let Ok(mut list) = jobs.lock() {
                    if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                        j.progress = p;
                    }
                }
            };

            let download_result = client.download_rom(rom_id, &temp_path, on_progress).await;
            if download_result.is_err() {
                let _ = tokio::fs::remove_file(&temp_path).await;
            }
            match download_result {
                Ok(()) => {
                    match finalize_download(&temp_path, &final_path).await {
                        Ok(FinalizeResult::Done) => {
                            if let Ok(mut list) = jobs.lock() {
                                if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                    j.status = DownloadStatus::Done;
                                    j.progress = 1.0;
                                }
                            }
                        }
                        Ok(FinalizeResult::SkippedAlreadyExists) => {
                            if let Ok(mut list) = jobs.lock() {
                                if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                    j.status = DownloadStatus::SkippedAlreadyExists;
                                    j.progress = 1.0;
                                }
                            }
                        }
                        Err(err) => {
                            let _ = tokio::fs::remove_file(&temp_path).await;
                            if let Ok(mut list) = jobs.lock() {
                                if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                                    j.status = DownloadStatus::FinalizeFailed(err.to_string());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Ok(mut list) = jobs.lock() {
                        if let Some(j) = list.iter_mut().find(|j| j.id == job_id) {
                            j.status = DownloadStatus::Error(e.to_string());
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FinalizeResult {
    Done,
    SkippedAlreadyExists,
}

async fn finalize_download(temp_path: &Path, final_path: &Path) -> Result<FinalizeResult> {
    if final_path.exists() {
        let _ = tokio::fs::remove_file(temp_path).await;
        return Ok(FinalizeResult::SkippedAlreadyExists);
    }

    match tokio::fs::rename(temp_path, final_path).await {
        Ok(()) => Ok(FinalizeResult::Done),
        Err(rename_err) if is_cross_device_rename_error(&rename_err) => {
            tokio::fs::copy(temp_path, final_path).await.with_context(|| {
                format!(
                    "Could not copy temp ROM {} to final destination {}",
                    temp_path.display(),
                    final_path.display()
                )
            })?;
            let file = tokio::fs::File::open(final_path).await.with_context(|| {
                format!("Could not open finalized ROM for sync: {}", final_path.display())
            })?;
            file.sync_all().await.with_context(|| {
                format!("Could not sync finalized ROM to disk: {}", final_path.display())
            })?;
            tokio::fs::remove_file(temp_path).await.with_context(|| {
                format!("Could not remove temp ROM after copy: {}", temp_path.display())
            })?;
            Ok(FinalizeResult::Done)
        }
        Err(rename_err) => Err(anyhow!(
            "Could not move temp ROM {} to final destination {}: {}",
            temp_path.display(),
            final_path.display(),
            rename_err
        )),
    }
}

fn is_cross_device_rename_error(err: &std::io::Error) -> bool {
    matches!(err.raw_os_error(), Some(18) | Some(17))
}

fn sanitized_final_filename(fs_name: &str, rom_id: u64) -> String {
    let sanitized = utils::sanitize_filename(fs_name);
    if sanitized.trim().is_empty() {
        format!("rom-{rom_id}.zip")
    } else {
        sanitized
    }
}

#[cfg(test)]
fn final_download_path_for_rom(roms_dir: &Path, rom: &Rom) -> PathBuf {
    let platform_slug = rom
        .platform_fs_slug
        .clone()
        .or_else(|| rom.platform_slug.clone())
        .unwrap_or_else(|| format!("platform-{}", rom.platform_id));
    let console_dir = roms_dir.join(utils::sanitize_filename(&platform_slug));
    console_dir.join(sanitized_final_filename(&rom.fs_name, rom.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::types::Rom;

    fn rom_fixture_with_platform(platform_fs_slug: Option<&str>, fs_name: &str) -> Rom {
        Rom {
            id: 42,
            platform_id: 7,
            platform_slug: Some("nintendo-switch".to_string()),
            platform_fs_slug: platform_fs_slug.map(ToString::to_string),
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: fs_name.to_string(),
            fs_name_no_tags: "game".to_string(),
            fs_name_no_ext: "game".to_string(),
            fs_extension: "zip".to_string(),
            fs_path: "/game.zip".to_string(),
            fs_size_bytes: 1,
            name: "Game".to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            is_unidentified: false,
            is_identified: true,
        }
    }

    #[test]
    fn unique_zip_path_skips_existing_files() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("romm-dl-test-{ts}"));
        std::fs::create_dir_all(&dir).unwrap();
        let p1 = dir.join("game.zip");
        std::fs::File::create(&p1).unwrap().write_all(b"x").unwrap();
        let p2 = unique_zip_path(&dir, "game");
        assert_eq!(p2.file_name().unwrap(), "game__2.zip");
        let _ = std::fs::remove_file(&p1);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn resolve_download_directory_rejects_empty_configured_path() {
        let err = resolve_download_directory_from_inputs(Some("   "), None)
            .expect_err("empty configured path should be rejected");
        assert!(
            err.to_string().contains("cannot be empty"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn resolve_download_directory_creates_missing_nested_directory() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("romm-dl-resolve-{ts}"));
        let nested = base.join("a").join("b").join("c");
        let nested_str = nested.to_string_lossy().to_string();

        let resolved = resolve_download_directory_from_inputs(Some(&nested_str), None)
            .expect("expected missing directory to be created");

        assert!(resolved.is_dir(), "resolved path must be a directory");
        assert!(nested.is_dir(), "nested path should be created");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn resolve_download_directory_fails_when_target_is_a_file() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("romm-dl-file-target-{ts}"));
        std::fs::create_dir_all(&base).expect("create base dir");
        let file_path = base.join("not-a-dir.txt");
        std::fs::write(&file_path, b"x").expect("create file");
        let input = file_path.to_string_lossy().to_string();

        let err = resolve_download_directory_from_inputs(Some(&input), None)
            .expect_err("file target must fail");
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err:#}"
        );

        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn resolve_download_directory_env_override_takes_precedence() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let configured = std::env::temp_dir().join(format!("romm-dl-configured-{ts}"));
        let env_dir = std::env::temp_dir().join(format!("romm-dl-env-{ts}"));
        let configured_str = configured.to_string_lossy().to_string();
        let env_str = env_dir.to_string_lossy().to_string();

        let resolved = resolve_download_directory_from_inputs(Some(&configured_str), Some(&env_str))
            .expect("env override should be used");

        assert_eq!(resolved, env_dir);
        assert!(env_dir.is_dir(), "env directory should be created");
        assert!(
            !configured.is_dir(),
            "configured path should be ignored when env override is set"
        );
        let _ = std::fs::remove_dir_all(&env_dir);
    }

    #[test]
    fn final_download_path_uses_console_folder_and_original_file_name() {
        let rom = rom_fixture_with_platform(Some("switch"), "Zelda (USA).xci");
        let base = PathBuf::from("/roms");
        let out = final_download_path_for_rom(&base, &rom);
        assert_eq!(out, PathBuf::from("/roms/switch/Zelda _USA_.xci"));
    }

    #[tokio::test]
    async fn finalize_download_skips_when_final_exists() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("romm-finalize-skip-{ts}"));
        std::fs::create_dir_all(&base).unwrap();
        let temp = base.join("temp.part");
        let final_path = base.join("final.zip");
        std::fs::write(&temp, b"temp").unwrap();
        std::fs::write(&final_path, b"existing").unwrap();

        let result = finalize_download(&temp, &final_path).await.unwrap();
        assert_eq!(result, super::FinalizeResult::SkippedAlreadyExists);
        assert!(
            !temp.exists(),
            "temp file should be removed when final destination exists"
        );

        let _ = std::fs::remove_file(&final_path);
        let _ = std::fs::remove_dir_all(&base);
    }
}
