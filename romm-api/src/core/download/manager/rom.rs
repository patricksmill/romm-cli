use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::client::RommClient;
use crate::config::RomsLayoutConfig;
use crate::core::extras::{build_base_rom_file_targets, DownloadAssetKind, DownloadTarget};
use crate::core::utils;
use crate::error::DownloadError;
use crate::types::Rom;

use super::super::job::{DownloadJob, DownloadStatus};
use super::super::paths::{resolve_console_roms_dir, resolve_download_directory};
use super::super::transfer::{download_target_with_fallback, prepare_download_target_destination};
use super::DownloadManager;

struct RomDownloadTask {
    client: RommClient,
    jobs: Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    rom_id: u64,
    console_dir: PathBuf,
    legacy_destination: PathBuf,
    base_targets: Vec<DownloadTarget>,
}

impl DownloadManager {
    pub fn start_download(
        &self,
        rom: &Rom,
        client: RommClient,
        layout: &RomsLayoutConfig,
        configured_download_dir: Option<&str>,
    ) -> Result<(), DownloadError> {
        let platform = rom
            .platform_display_name
            .as_deref()
            .or(rom.platform_custom_name.as_deref())
            .unwrap_or("—")
            .to_string();

        let job = DownloadJob::new(rom.id, rom.name.clone(), platform);
        let job_id = job.id;
        let rom_id = rom.id;
        let rom_for_targets = rom.clone();
        let layout = layout.clone();
        match self.jobs.lock() {
            Ok(mut jobs) => jobs.push(job),
            Err(err) => {
                eprintln!("warning: download job list lock poisoned: {}", err);
                return Err(DownloadError::JobListPoisoned(err.to_string()));
            }
        }

        let save_dir = resolve_download_directory(configured_download_dir)?;
        let console_dir = resolve_console_roms_dir(&layout, &save_dir, rom)?;
        let legacy_destination = console_dir.join(sanitized_rom_filename(&rom.fs_name, rom.id));
        let base_targets = build_base_rom_file_targets(&rom_for_targets, &layout, &save_dir)?;
        let task = RomDownloadTask {
            client,
            jobs: self.jobs.clone(),
            job_id,
            rom_id,
            console_dir,
            legacy_destination,
            base_targets,
        };
        tokio::spawn(run_rom_download_task(task, layout, save_dir));
        Ok(())
    }
}

async fn run_rom_download_task(task: RomDownloadTask, layout: RomsLayoutConfig, save_dir: PathBuf) {
    if let Err(err) = tokio::fs::create_dir_all(&task.console_dir).await {
        set_job_status(
            &task.jobs,
            task.job_id,
            DownloadStatus::Error(format!(
                "Could not create console directory {}: {err}",
                task.console_dir.display()
            )),
        );
        return;
    }

    // ROMs from the library list endpoint may have an empty `files` vec because
    // list responses omit per-file records. Always resolve the full file list by
    // preferring what we already have, then falling back to a detail fetch.
    // If the detail also returns no files, this is a legacy single-file ROM.
    // The aggregate endpoint returns a ZIP, so keep the destination extension
    // explicit instead of using the original ROM extension.
    let targets = if !task.base_targets.is_empty() {
        task.base_targets
    } else if task.legacy_destination.exists() {
        finish_job(
            &task.jobs,
            task.job_id,
            DownloadStatus::SkippedAlreadyExists,
        );
        return;
    } else {
        match fetch_base_targets_from_detail(
            &task.client,
            &task.jobs,
            task.job_id,
            task.rom_id,
            &layout,
            &save_dir,
        )
        .await
        {
            Some(t) => t,
            None => return, // error already recorded
        }
    };

    download_base_targets(&task.client, &task.jobs, task.job_id, &targets).await;
}

/// Fetches the full ROM detail (which includes file records) and builds
/// per-file download targets.
///
/// Returns `None` if the API call fails or targets cannot be built; the error is
/// recorded in the job before returning.
async fn fetch_base_targets_from_detail(
    client: &RommClient,
    jobs: &Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    rom_id: u64,
    layout: &RomsLayoutConfig,
    save_dir: &std::path::Path,
) -> Option<Vec<DownloadTarget>> {
    use crate::endpoints::roms::GetRom;

    let detailed_rom = match client.call(&GetRom { id: rom_id }).await {
        Ok(r) => r,
        Err(e) => {
            set_job_status(
                jobs,
                job_id,
                DownloadStatus::Error(format!("Failed to fetch ROM detail: {e}")),
            );
            return None;
        }
    };

    match build_base_rom_file_targets(&detailed_rom, layout, save_dir) {
        Ok(targets) if !targets.is_empty() => Some(targets),
        Ok(_) if detailed_rom.files.is_empty() => {
            match legacy_zip_download_fallback(&detailed_rom, layout, save_dir) {
                Ok(LegacyZipFallback::SkipAlreadyExists) => {
                    finish_job(jobs, job_id, DownloadStatus::SkippedAlreadyExists);
                    None
                }
                Ok(LegacyZipFallback::Download(target)) => Some(vec![target]),
                Err(e) => {
                    set_job_status(
                        jobs,
                        job_id,
                        DownloadStatus::Error(format!(
                            "Failed to build fallback download target: {e}"
                        )),
                    );
                    None
                }
            }
        }
        Ok(_) => {
            set_job_status(
                jobs,
                job_id,
                DownloadStatus::Error(
                    "No base game file records found for this ROM. \
                     Re-scan the library in RomM to populate base file metadata."
                        .to_string(),
                ),
            );
            None
        }
        Err(e) => {
            set_job_status(
                jobs,
                job_id,
                DownloadStatus::Error(format!("Failed to build download targets: {e}")),
            );
            None
        }
    }
}

enum LegacyZipFallback {
    SkipAlreadyExists,
    Download(DownloadTarget),
}

fn legacy_zip_download_fallback(
    rom: &Rom,
    layout: &RomsLayoutConfig,
    save_dir: &std::path::Path,
) -> Result<LegacyZipFallback, DownloadError> {
    let console_dir = resolve_console_roms_dir(layout, save_dir, rom)?;
    let original_destination = console_dir.join(sanitized_rom_filename(&rom.fs_name, rom.id));
    if original_destination.exists() {
        return Ok(LegacyZipFallback::SkipAlreadyExists);
    }

    let destination = reserve_unique_zip_path(&console_dir, &format!("rom_{}", rom.id))?;
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download.zip")
        .to_string();

    Ok(LegacyZipFallback::Download(DownloadTarget {
        kind: DownloadAssetKind::RomArchive,
        title: rom.fs_name.clone(),
        source_url: "/api/roms/download".to_string(),
        source_query: vec![
            ("rom_ids".to_string(), rom.id.to_string()),
            ("filename".to_string(), filename),
        ],
        destination,
        expected_size_bytes: None,
    }))
}

fn sanitized_rom_filename(fs_name: &str, rom_id: u64) -> String {
    let sanitized = utils::sanitize_filename(fs_name);
    if sanitized.trim().is_empty() {
        format!("rom-{rom_id}.zip")
    } else {
        sanitized
    }
}

fn reserve_unique_zip_path(dir: &std::path::Path, stem: &str) -> Result<PathBuf, DownloadError> {
    std::fs::create_dir_all(dir).map_err(|e| DownloadError::IoContext {
        context: format!("Could not create fallback ZIP directory {}", dir.display()),
        source: e,
    })?;

    let mut n = 1u32;
    loop {
        let name = if n == 1 {
            format!("{stem}.zip")
        } else {
            format!("{stem}__{n}.zip")
        };
        let candidate = dir.join(name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(_) => return Ok(candidate),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                n = n.saturating_add(1);
            }
            Err(source) => {
                return Err(DownloadError::IoContext {
                    context: format!("Could not reserve fallback ZIP {}", candidate.display()),
                    source,
                });
            }
        }
    }
}

async fn download_base_targets(
    client: &RommClient,
    jobs: &Arc<Mutex<Vec<DownloadJob>>>,
    job_id: usize,
    base_targets: &[DownloadTarget],
) {
    let total_targets = base_targets.len() as f64;
    for (idx, target) in base_targets.iter().enumerate() {
        let progress_jobs = jobs.clone();
        let mut progress = move |received: u64, total: u64| {
            let file_ratio = if total > 0 {
                received as f64 / total as f64
            } else {
                0.0
            };
            let total_ratio = ((idx as f64) + file_ratio) / total_targets;
            set_job_progress(&progress_jobs, job_id, total_ratio.min(1.0));
        };

        match prepare_download_target_destination(target).await {
            Ok(true) => {
                progress(
                    target.expected_size_bytes.unwrap_or(0),
                    target.expected_size_bytes.unwrap_or(0),
                );
                continue;
            }
            Ok(false) => {}
            Err(err) => {
                set_job_status(jobs, job_id, DownloadStatus::Error(err.to_string()));
                return;
            }
        }

        if let Err(final_err) =
            download_target_with_fallback(client, target, |_, _| false, &mut progress).await
        {
            set_job_status(jobs, job_id, DownloadStatus::Error(final_err.to_string()));
            return;
        }
    }
    finish_job(jobs, job_id, DownloadStatus::Done);
}

fn update_download_job<F>(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, update: F)
where
    F: FnOnce(&mut DownloadJob),
{
    if let Ok(mut list) = jobs.lock() {
        if let Some(job) = list.iter_mut().find(|job| job.id == job_id) {
            update(job);
        }
    }
}

fn set_job_progress(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, progress: f64) {
    update_download_job(jobs, job_id, |job| {
        job.progress = progress;
    });
}

fn set_job_status(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, status: DownloadStatus) {
    update_download_job(jobs, job_id, |job| {
        job.status = status;
    });
}

fn finish_job(jobs: &Arc<Mutex<Vec<DownloadJob>>>, job_id: usize, status: DownloadStatus) {
    update_download_job(jobs, job_id, |job| {
        job.status = status;
        job.progress = 1.0;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::{default_theme_id, Config};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn rom_fixture() -> Rom {
        Rom {
            id: 42,
            platform_id: 7,
            platform_slug: Some("snes".to_string()),
            platform_fs_slug: Some("snes".to_string()),
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: "Legacy Game.sfc".to_string(),
            fs_name_no_tags: "Legacy Game".to_string(),
            fs_name_no_ext: "Legacy Game".to_string(),
            fs_extension: "sfc".to_string(),
            fs_path: "/Legacy Game.sfc".to_string(),
            fs_size_bytes: 9,
            name: "Legacy Game".to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            has_manual: false,
            path_manual: None,
            url_manual: None,
            is_unidentified: false,
            is_identified: true,
            files: Vec::new(),
            ra_id: None,
            merged_ra_metadata: None,
        }
    }

    fn test_client(base_url: &str) -> RommClient {
        RommClient::new(
            &Config {
                base_url: base_url.to_string(),
                download_dir: ".".to_string(),
                use_https: false,
                auth: None,
                extras_defaults: Default::default(),
                save_sync: Default::default(),
                roms_layout: Default::default(),
                theme: default_theme_id(),
                tui_layout: Default::default(),
            },
            false,
        )
        .expect("client")
    }

    #[tokio::test]
    async fn falls_back_to_zip_download_when_detail_has_no_file_records() {
        let server = MockServer::start().await;
        let rom = rom_fixture();
        let temp_dir = std::env::temp_dir().join(format!(
            "romm-manager-legacy-fallback-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        Mock::given(method("GET"))
            .and(path("/api/roms/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&rom))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/roms/download"))
            .and(query_param("rom_ids", "42"))
            .and(query_param("filename", "rom_42.zip"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"zip-bytes".to_vec()))
            .expect(1)
            .mount(&server)
            .await;

        let job = DownloadJob::new(rom.id, rom.name.clone(), "SNES".to_string());
        let job_id = job.id;
        let jobs = Arc::new(Mutex::new(vec![job]));
        let client = test_client(&server.uri());
        let targets = fetch_base_targets_from_detail(
            &client,
            &jobs,
            job_id,
            rom.id,
            &RomsLayoutConfig::default(),
            &temp_dir,
        )
        .await
        .expect("fallback target");

        download_base_targets(&client, &jobs, job_id, &targets).await;

        let status = jobs.lock().expect("jobs lock")[0].status.clone();
        assert!(
            matches!(status, DownloadStatus::Done),
            "expected done status, got {status:?}"
        );
        assert_eq!(
            tokio::fs::read(temp_dir.join("snes").join("rom_42.zip"))
                .await
                .unwrap(),
            b"zip-bytes"
        );

        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }

    #[tokio::test]
    async fn does_not_zip_fallback_when_detail_has_only_non_base_file_records() {
        let server = MockServer::start().await;
        let mut rom = rom_fixture();
        rom.files = vec![crate::types::RomFile {
            id: 9,
            rom_id: rom.id,
            file_name: "Legacy Game update.sfc".to_string(),
            file_path: "/Legacy Game update.sfc".to_string(),
            file_size_bytes: 9,
            category: Some(crate::types::RomFileCategory::Update),
        }];
        let temp_dir = std::env::temp_dir().join(format!(
            "romm-manager-non-base-files-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        Mock::given(method("GET"))
            .and(path("/api/roms/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&rom))
            .expect(1)
            .mount(&server)
            .await;

        let job = DownloadJob::new(rom.id, rom.name.clone(), "SNES".to_string());
        let job_id = job.id;
        let jobs = Arc::new(Mutex::new(vec![job]));
        let client = test_client(&server.uri());

        let targets = fetch_base_targets_from_detail(
            &client,
            &jobs,
            job_id,
            rom.id,
            &RomsLayoutConfig::default(),
            &temp_dir,
        )
        .await;

        assert!(
            targets.is_none(),
            "non-base files must not use ZIP fallback"
        );
        let status = jobs.lock().expect("jobs lock")[0].status.clone();
        assert!(
            matches!(status, DownloadStatus::Error(ref message) if message.contains("base game")),
            "expected base-game error, got {status:?}"
        );

        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }

    #[test]
    fn legacy_zip_fallback_reserves_unique_destinations() {
        let rom = rom_fixture();
        let temp_dir = std::env::temp_dir().join(format!(
            "romm-manager-reserved-fallback-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let first =
            legacy_zip_download_fallback(&rom, &RomsLayoutConfig::default(), &temp_dir).unwrap();
        let second =
            legacy_zip_download_fallback(&rom, &RomsLayoutConfig::default(), &temp_dir).unwrap();

        let LegacyZipFallback::Download(first) = first else {
            panic!("expected first fallback target");
        };
        let LegacyZipFallback::Download(second) = second else {
            panic!("expected second fallback target");
        };
        assert_eq!(first.destination.file_name().unwrap(), "rom_42.zip");
        assert_eq!(second.destination.file_name().unwrap(), "rom_42__2.zip");
        assert!(first.destination.exists());
        assert!(second.destination.exists());

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
