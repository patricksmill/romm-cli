//! ROM/save path resolution and zip extraction helpers.

use std::path::{Path, PathBuf};

use crate::config::RomsLayoutConfig;
use crate::config::{resolved_save_dir, Config, SaveSyncConfig};
use crate::core::utils;
use crate::error::DownloadError;
use crate::types::Rom;
use std::fs::File;
use zip::ZipArchive;

/// Directory for ROM storage (`ROMM_ROMS_DIR`, `ROMM_DOWNLOAD_DIR`, or configured path).
pub fn resolve_download_directory(
    configured_download_dir: Option<&str>,
) -> Result<PathBuf, DownloadError> {
    let env_override = std::env::var("ROMM_ROMS_DIR")
        .ok()
        .or_else(|| std::env::var("ROMM_DOWNLOAD_DIR").ok());
    resolve_download_directory_from_inputs(configured_download_dir, env_override.as_deref())
}

/// Validate configured download path without env override fallback.
pub fn validate_configured_download_directory(
    configured_download_dir: &str,
) -> Result<PathBuf, DownloadError> {
    resolve_download_directory_from_inputs(Some(configured_download_dir), None)
}

/// Backward-compatible default used by legacy CLI download code.
pub fn download_directory() -> PathBuf {
    std::env::var("ROMM_ROMS_DIR")
        .or_else(|_| std::env::var("ROMM_DOWNLOAD_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./downloads"))
}

pub(crate) fn resolve_download_directory_from_inputs(
    configured_download_dir: Option<&str>,
    env_override: Option<&str>,
) -> Result<PathBuf, DownloadError> {
    let raw = env_override
        .or(configured_download_dir)
        .map(str::trim)
        .ok_or(DownloadError::PathNotConfigured)?;

    if raw.is_empty() {
        return Err(DownloadError::RomsDirEmpty);
    }

    let input_path = PathBuf::from(raw);
    let normalized = if input_path.is_relative() {
        std::env::current_dir()
            .map_err(|e| DownloadError::IoContext {
                context: "Could not resolve current working directory".into(),
                source: e,
            })?
            .join(input_path)
    } else {
        input_path
    };

    if normalized.exists() && !normalized.is_dir() {
        return Err(DownloadError::InvalidRomsDir {
            path: normalized.display().to_string(),
        });
    }

    std::fs::create_dir_all(&normalized).map_err(|e| DownloadError::IoContext {
        context: format!(
            "Could not create download directory {}",
            normalized.display()
        ),
        source: e,
    })?;

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
        .map_err(|e| DownloadError::IoContext {
            context: format!("ROMs directory is not writable: {}", normalized.display()),
            source: e,
        })?;
    drop(probe);
    let _ = std::fs::remove_file(&probe_path);

    Ok(normalized)
}

/// Filesystem slug used for auto-mode console subfolders.
pub fn platform_download_slug(rom: &Rom) -> String {
    rom.platform_fs_slug
        .clone()
        .or_else(|| rom.platform_slug.clone())
        .unwrap_or_else(|| format!("platform-{}", rom.platform_id))
}

fn auto_console_roms_dir(base_download_dir: &Path, rom: &Rom) -> PathBuf {
    base_download_dir.join(utils::sanitize_filename(&platform_download_slug(rom)))
}

/// Resolve the directory where ROM files for `rom` should be stored.
pub fn resolve_console_roms_dir(
    layout: &RomsLayoutConfig,
    base_download_dir: &Path,
    rom: &Rom,
) -> Result<PathBuf, DownloadError> {
    if let Some(raw) = layout
        .platform_dirs
        .get(&rom.platform_id)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        validate_configured_download_directory(raw)
    } else {
        Ok(auto_console_roms_dir(base_download_dir, rom))
    }
}

fn save_platform_slug(
    platform_id: u64,
    platform_fs_slug: Option<&str>,
    platform_slug: Option<&str>,
) -> String {
    utils::sanitize_filename(
        platform_fs_slug
            .or(platform_slug)
            .unwrap_or(&format!("platform-{platform_id}")),
    )
}

fn auto_console_save_dir(
    base_save_dir: &Path,
    platform_id: u64,
    platform_fs_slug: Option<&str>,
    platform_slug: Option<&str>,
) -> PathBuf {
    base_save_dir.join(save_platform_slug(
        platform_id,
        platform_fs_slug,
        platform_slug,
    ))
}

/// Resolve the directory where save files for a console should be stored.
pub fn resolve_console_save_dir(
    save_sync: &SaveSyncConfig,
    base_save_dir: &Path,
    platform_id: u64,
    platform_fs_slug: Option<&str>,
    platform_slug: Option<&str>,
) -> Result<PathBuf, DownloadError> {
    if let Some(raw) = save_sync
        .platform_dirs
        .get(&platform_id)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        validate_configured_download_directory(raw)
    } else {
        Ok(auto_console_save_dir(
            base_save_dir,
            platform_id,
            platform_fs_slug,
            platform_slug,
        ))
    }
}

fn safe_game_path_segment(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        "game".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Resolve the directory where a specific game's saves should be downloaded.
pub fn resolve_game_save_dir(config: &Config, rom: &Rom) -> Result<PathBuf, DownloadError> {
    let base = resolved_save_dir(config);
    let console_dir = resolve_console_save_dir(
        &config.save_sync,
        &base,
        rom.platform_id,
        rom.platform_fs_slug.as_deref(),
        rom.platform_slug.as_deref(),
    )?;
    Ok(console_dir.join(safe_game_path_segment(&rom.name)))
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

/// Extract a ZIP archive into `destination_dir`.
pub fn extract_zip_archive(zip_path: &Path, destination_dir: &Path) -> Result<(), DownloadError> {
    let zip_path = zip_path.to_path_buf();
    let destination_dir = destination_dir.to_path_buf();
    std::fs::create_dir_all(&destination_dir).map_err(|e| DownloadError::IoContext {
        context: format!(
            "Could not create extraction directory {}",
            destination_dir.display()
        ),
        source: e,
    })?;

    let file = File::open(&zip_path).map_err(|e| DownloadError::IoContext {
        context: format!("Could not open zip archive {}", zip_path.display()),
        source: e,
    })?;
    let mut archive = ZipArchive::new(file).map_err(|e| DownloadError::IoContext {
        context: format!("Invalid ZIP archive {}", zip_path.display()),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })?;
    archive
        .extract(&destination_dir)
        .map_err(|e| DownloadError::IoContext {
            context: format!(
                "Could not extract archive into {}",
                destination_dir.display()
            ),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;
    Ok(())
}
