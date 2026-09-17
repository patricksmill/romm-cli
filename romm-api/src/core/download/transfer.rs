//! HTTP download, URL fallback, and finalize helpers.

use crate::client::RommClient;
use crate::core::extras::DownloadTarget;
use crate::error::DownloadError;

pub async fn prepare_download_target_destination(
    target: &DownloadTarget,
) -> Result<bool, DownloadError> {
    let Some(expected_size) = target.expected_size_bytes else {
        return Ok(false);
    };
    if expected_size == 0 {
        return Ok(false);
    }

    let Ok(metadata) = tokio::fs::metadata(&target.destination).await else {
        return Ok(false);
    };
    let current_size = metadata.len();
    if current_size == expected_size {
        return Ok(true);
    }
    if current_size > expected_size {
        tokio::fs::remove_file(&target.destination)
            .await
            .map_err(|e| DownloadError::IoContext {
                context: format!(
                    "remove oversized stale download {} ({} > {} bytes)",
                    target.destination.display(),
                    current_size,
                    expected_size
                ),
                source: e,
            })?;
    }
    Ok(false)
}

/// Download a target, trying alternate RomM file URL shapes on HTTP 404.
pub async fn download_target_with_fallback<F, C>(
    client: &RommClient,
    target: &DownloadTarget,
    mut is_cancelled: C,
    on_progress: &mut F,
) -> Result<(), DownloadError>
where
    F: FnMut(u64, u64) + Send,
    C: FnMut(u64, u64) -> bool + Send,
{
    let urls = candidate_download_urls(target);
    let mut last_err: Option<DownloadError> = None;
    for url in urls {
        match client
            .download_url_with_query_with_cancel(
                &url,
                &target.source_query,
                &target.destination,
                &mut is_cancelled,
                on_progress,
            )
            .await
        {
            Ok(()) => return Ok(()),
            Err(err) => {
                if !err.is_not_found() {
                    return Err(err);
                }
                last_err = Some(err);
            }
        }
    }
    Err(last_err.unwrap_or(DownloadError::FailedWithoutDetails))
}

pub(crate) fn candidate_download_urls(target: &DownloadTarget) -> Vec<String> {
    let mut out = vec![target.source_url.clone()];
    if let Some((file_id, file_name)) = parse_current_rom_file_content_path(&target.source_url) {
        out.push(format!("/api/romsfiles/{file_id}/content/{file_name}"));
        out.push(format!("/api/roms/files/{file_id}/content/{file_name}"));
    } else if let Some((file_id, file_name)) = parse_romsfiles_path(&target.source_url) {
        out.push(format!("/api/roms/{file_id}/files/content/{file_name}"));
        out.push(format!("/api/roms/files/{file_id}/content/{file_name}"));
    } else if let Some((file_id, file_name)) = parse_legacy_roms_files_path(&target.source_url) {
        out.push(format!("/api/roms/{file_id}/files/content/{file_name}"));
        out.push(format!("/api/romsfiles/{file_id}/content/{file_name}"));
    }
    dedupe_preserve_order(out)
}

fn parse_current_rom_file_content_path(url: &str) -> Option<(String, String)> {
    let prefix = "/api/roms/";
    let marker = "/files/content/";
    let rest = url.strip_prefix(prefix)?;
    let (id, name) = rest.split_once(marker)?;
    Some((id.to_string(), name.to_string()))
}

fn parse_romsfiles_path(url: &str) -> Option<(String, String)> {
    let prefix = "/api/romsfiles/";
    let marker = "/content/";
    let rest = url.strip_prefix(prefix)?;
    let (id, name) = rest.split_once(marker)?;
    Some((id.to_string(), name.to_string()))
}

fn parse_legacy_roms_files_path(url: &str) -> Option<(String, String)> {
    let prefix = "/api/roms/files/";
    let marker = "/content/";
    let rest = url.strip_prefix(prefix)?;
    let (id, name) = rest.split_once(marker)?;
    Some((id.to_string(), name.to_string()))
}

fn dedupe_preserve_order(urls: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for u in urls {
        if seen.insert(u.clone()) {
            out.push(u);
        }
    }
    out
}

#[cfg(test)]
use crate::core::utils;

#[cfg(test)]
pub(crate) fn sanitized_final_filename(fs_name: &str, rom_id: u64) -> String {
    let sanitized = utils::sanitize_filename(fs_name);
    if sanitized.trim().is_empty() {
        format!("rom-{rom_id}.zip")
    } else {
        sanitized
    }
}

#[cfg(test)]
pub(crate) fn final_download_path_for_rom(
    roms_dir: &std::path::Path,
    rom: &crate::types::Rom,
) -> std::path::PathBuf {
    let platform_slug = rom
        .platform_fs_slug
        .clone()
        .or_else(|| rom.platform_slug.clone())
        .unwrap_or_else(|| format!("platform-{}", rom.platform_id));
    let console_dir = roms_dir.join(utils::sanitize_filename(&platform_slug));
    console_dir.join(sanitized_final_filename(&rom.fs_name, rom.id))
}
