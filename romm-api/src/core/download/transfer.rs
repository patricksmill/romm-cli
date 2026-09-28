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
    if current_size != expected_size {
        tokio::fs::remove_file(&target.destination)
            .await
            .map_err(|e| DownloadError::IoContext {
                context: format!(
                    "remove stale download {} ({} != {} bytes)",
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
            Ok(()) => return verify_download_target_size(target).await,
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

async fn verify_download_target_size(target: &DownloadTarget) -> Result<(), DownloadError> {
    let Some(expected_size) = target.expected_size_bytes else {
        return Ok(());
    };
    let actual_size = tokio::fs::metadata(&target.destination)
        .await
        .map_err(|e| DownloadError::IoContext {
            context: format!(
                "verify downloaded file size {}",
                target.destination.display()
            ),
            source: e,
        })?
        .len();
    if actual_size == expected_size {
        return Ok(());
    }

    let _ = tokio::fs::remove_file(&target.destination).await;
    Err(DownloadError::Unexpected(format!(
        "downloaded file {} is {actual_size} bytes, expected {expected_size} bytes",
        target.destination.display()
    )))
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
use crate::core::path_segment::{JoinSegment, PathSegment};

#[cfg(test)]
pub(crate) fn sanitized_final_filename(fs_name: &str, rom_id: u64) -> PathSegment {
    PathSegment::sanitize(fs_name, &format!("rom-{rom_id}.zip"))
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
    let fallback = format!("platform-{}", rom.platform_id);
    let console_dir = roms_dir.join_segment(&PathSegment::sanitize(&platform_slug, &fallback));
    console_dir.join_segment(&sanitized_final_filename(&rom.fs_name, rom.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::extras::{DownloadAssetKind, DownloadTarget};

    fn temp_path(label: &str) -> std::path::PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("romm-download-{label}-{}-{ts}", std::process::id()))
    }

    fn target(destination: std::path::PathBuf, expected_size_bytes: Option<u64>) -> DownloadTarget {
        DownloadTarget {
            kind: DownloadAssetKind::RomFile,
            title: "file".into(),
            source_url: "/api/roms/1/files/content/file.bin".into(),
            source_query: Vec::new(),
            destination,
            expected_size_bytes,
        }
    }

    #[tokio::test]
    async fn prepare_target_removes_undersized_existing_file_before_download() {
        let path = temp_path("undersized");
        std::fs::write(&path, b"OLD").expect("write stale partial");
        let target = target(path.clone(), Some(7));

        let already_present = prepare_download_target_destination(&target)
            .await
            .expect("prepare target");

        assert!(!already_present);
        assert!(
            !path.exists(),
            "undersized stale file must be removed so the next download cannot append to it"
        );
    }
}
