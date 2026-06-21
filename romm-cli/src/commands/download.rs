use crate::cli_presentation::{format_command_error, CliPresentation};
use clap::{Args, Subcommand, ValueEnum};
use dialoguer::Confirm;
use indicatif::ProgressBar;
use romm_api::error::{DownloadError, RommError};
use serde::Serialize;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

use romm_api::client::RommClient;
use romm_api::config::{load_config, RomsLayoutConfig};
use romm_api::core::download::{
    download_target_with_fallback, extract_zip_archive, prepare_download_target_destination,
    resolve_console_roms_dir, resolve_download_directory, unique_zip_path,
};
use romm_api::core::extras::{
    build_base_rom_file_targets, build_extras_targets, build_update_dlc_targets_for_rom,
    DownloadTarget,
};
use romm_api::core::interrupt::{
    cancelled_download_error, is_cancelled_download, is_cancelled_error, InterruptContext,
};
use romm_api::core::resolve::resolve_platform_id;
use romm_api::core::utils;
use romm_api::endpoints::roms::{GetRom, GetRoms};
/// Maximum number of concurrent download connections.
const DEFAULT_CONCURRENCY: usize = 4;

const DOWNLOAD_PROGRESS_PLAIN: &str =
    "[{elapsed_precise}] {bar:40} {bytes}/{total_bytes} ({eta}) {msg}";
const DOWNLOAD_PROGRESS_COLOR: &str =
    "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}";

#[derive(Default, Serialize)]
struct DownloadJsonSummary {
    succeeded: u32,
    failed: u32,
    cancelled: u32,
    paths: Vec<String>,
}

fn parse_nonzero_usize(value: &str) -> std::result::Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|err| format!("invalid number: {err}"))?;
    if parsed == 0 {
        Err("must be at least 1".to_string())
    } else {
        Ok(parsed)
    }
}

/// Download a ROM to the local filesystem with a progress bar.
#[derive(Args, Debug)]
#[command(after_help = "Examples:\n  \
      romm-cli download 42\n  \
      romm-cli download batch --platform gba --search-term zelda\n  \
      romm-cli download extras 42")]
pub struct DownloadCommand {
    /// ID of the ROM to download in single-ROM mode
    pub rom_id: Option<u64>,

    #[command(subcommand)]
    pub action: Option<DownloadAction>,

    /// Directory to save the ROM zip(s) to
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,

    /// Filter by platform slug or title (e.g. "3ds")
    #[arg(long, global = true)]
    pub platform: Option<String>,

    /// Filter by search term
    #[arg(long, global = true)]
    pub search_term: Option<String>,

    /// Maximum concurrent downloads (default: 4)
    #[arg(long, default_value_t = DEFAULT_CONCURRENCY, value_parser = parse_nonzero_usize, global = true)]
    pub jobs: usize,

    /// Extract each downloaded ZIP after download completes (batch mode only)
    #[arg(long, global = true)]
    pub extract: bool,

    /// Layout for extracted files when --extract is set (default: platform)
    #[arg(long, value_enum, default_value_t = ExtractLayout::Platform, global = true)]
    pub extract_layout: ExtractLayout,

    /// Delete ZIP files after successful extraction (batch mode only)
    #[arg(long, global = true)]
    pub delete_zip_after_extract: bool,

    /// Include updates and DLC after downloading the base game (single-ROM mode)
    #[arg(long, global = true)]
    pub with_extras: bool,

    /// Skip updates and DLC (single-ROM mode)
    #[arg(long, global = true)]
    pub no_extras: bool,

    /// Assume yes for extras prompt (single-ROM mode)
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DownloadAction {
    /// Download multiple ROMs matching filters
    #[command(visible_alias = "all")]
    Batch,
    /// Download covers, manuals, updates, and DLC for one game
    Extras(DownloadExtrasCommand),
}

#[derive(Args, Debug, Clone)]
pub struct DownloadExtrasCommand {
    /// ID of the ROM/game to download extras for
    pub rom_id: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ExtractLayout {
    /// Extract to `<output>/<platform_slug>/`
    Platform,
    /// Extract to `<output>/`
    Flat,
    /// Extract to `<output>/<platform_slug>/<rom_name>/`
    Rom,
}

async fn download_one(
    client: &RommClient,
    rom_id: u64,
    name: &str,
    save_path: &std::path::Path,
    pb: ProgressBar,
) -> Result<(), RommError> {
    pb.set_message(name.to_string());

    client
        .download_rom(rom_id, save_path, {
            let pb = pb.clone();
            move |received, total| {
                if pb.length() != Some(total) {
                    pb.set_length(total);
                }
                pb.set_position(received);
            }
        })
        .await?;

    pb.finish_with_message(format!("✓ {name}"));
    Ok(())
}

async fn download_target(
    client: &RommClient,
    target: &DownloadTarget,
    interrupt: &InterruptContext,
    pb: ProgressBar,
) -> Result<(), RommError> {
    pb.set_message(format!("{}: {}", target.kind.label(), target.title));

    let mut progress = {
        let pb = pb.clone();
        move |received, total| {
            if pb.length() != Some(total) {
                pb.set_length(total);
            }
            pb.set_position(received);
        }
    };

    if prepare_download_target_destination(target).await? {
        if let Some(expected_size) = target.expected_size_bytes {
            progress(expected_size, expected_size);
        }
        pb.finish_with_message(format!("✓ {}: {}", target.kind.label(), target.title));
        return Ok(());
    }

    download_target_with_fallback(
        client,
        target,
        |_, _| interrupt.is_cancelled(),
        &mut progress,
    )
    .await?;

    pb.finish_with_message(format!("✓ {}: {}", target.kind.label(), target.title));
    Ok(())
}

fn emit_download_summary(
    presentation: CliPresentation,
    summary: DownloadJsonSummary,
) -> Result<(), RommError> {
    if presentation.is_json() {
        presentation.emit_json(&summary)?;
    } else if summary.succeeded > 0 || summary.failed > 0 || summary.cancelled > 0 {
        presentation.emit_status(format!(
            "Download complete: {} succeeded, {} failed, {} cancelled.",
            summary.succeeded, summary.failed, summary.cancelled
        ));
    }
    Ok(())
}

pub async fn handle(
    cmd: DownloadCommand,
    client: &RommClient,
    presentation: CliPresentation,
    interrupt: Option<InterruptContext>,
) -> Result<(), RommError> {
    let interrupt = interrupt.unwrap_or_default();
    let config = load_config()?;
    let layout = config.roms_layout.clone();
    let output_dir = match cmd.output.clone() {
        Some(path) => path,
        None => resolve_download_directory(Some(config.download_dir.as_str()))?,
    };
    let action = cmd.action.clone();

    if cmd.with_extras && cmd.no_extras {
        return Err(RommError::Other(
            "--with-extras and --no-extras are mutually exclusive".into(),
        ));
    }

    // Ensure output directory exists.
    tokio::fs::create_dir_all(&output_dir).await.map_err(|e| {
        RommError::Download(DownloadError::IoContext {
            context: format!("create download dir {output_dir:?}"),
            source: e,
        })
    })?;

    if let Some(DownloadAction::Extras(extras)) = action.clone() {
        let summary = handle_extras(
            extras,
            client,
            interrupt,
            &layout,
            output_dir,
            cmd.jobs,
            presentation,
        )
        .await?;
        return emit_download_summary(presentation, summary);
    }

    // Determine if we are in batch mode.
    let is_batch = matches!(action, Some(DownloadAction::Batch));

    if is_batch {
        // ── Batch mode ─────────────────────────────────────────────────
        if cmd.platform.is_none() && cmd.search_term.is_none() {
            return Err(RommError::Other(
                "Batch download requires at least --platform or --search-term to scope the download"
                    .into(),
            ));
        }
        let resolved_platform_id = resolve_platform_id(client, cmd.platform.as_deref()).await?;

        let ep = GetRoms {
            search_term: cmd.search_term.clone(),
            platform_id: resolved_platform_id,
            collection_id: None,
            smart_collection_id: None,
            virtual_collection_id: None,
            limit: Some(9999),
            offset: None,
            ..Default::default()
        };

        let results = client.call(&ep).await?;

        if results.items.is_empty() {
            return emit_download_summary(presentation, DownloadJsonSummary::default());
        }

        presentation.emit_status(format!(
            "Found {} ROM(s). Starting download with {} concurrent connections...",
            results.items.len(),
            cmd.jobs
        ));

        let style = presentation.progress_style(DOWNLOAD_PROGRESS_PLAIN, DOWNLOAD_PROGRESS_COLOR);
        let mp = presentation.multi_progress();
        let semaphore = Arc::new(Semaphore::new(cmd.jobs));
        let mut handles = Vec::new();
        'enqueue: for rom in results.items {
            if interrupt.is_cancelled() {
                break 'enqueue;
            }
            let permit = semaphore.clone().acquire_owned().await.map_err(|_| {
                RommError::Other("download worker semaphore closed unexpectedly".into())
            })?;
            let client = client.clone();
            let base_dir = output_dir.clone();
            let layout = layout.clone();
            let interrupt = interrupt.clone();
            let pb = if let Some(ref mp) = mp {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(style.clone());
                pb
            } else {
                ProgressBar::hidden()
            };
            let name = rom.name.clone();
            let rom_id = rom.id;
            let console_dir = resolve_console_roms_dir(&layout, &base_dir, &rom)?;
            tokio::fs::create_dir_all(&console_dir).await.map_err(|e| {
                RommError::Download(DownloadError::IoContext {
                    context: format!("create console download dir {console_dir:?}"),
                    source: e,
                })
            })?;
            let platform_slug = rom
                .platform_fs_slug
                .clone()
                .or_else(|| rom.platform_slug.clone())
                .unwrap_or_else(|| format!("platform-{}", rom.platform_id));
            let base = utils::sanitize_filename(&rom.fs_name);
            let stem = base
                .rsplit_once('.')
                .map(|(s, _)| s.to_string())
                .unwrap_or(base.clone());
            let save_path = unique_zip_path(&console_dir, &stem);
            let extract = cmd.extract;
            let extract_layout = cmd.extract_layout;
            let delete_zip_after_extract = cmd.delete_zip_after_extract;

            handles.push(tokio::spawn(async move {
                let mut progress = {
                    let pb = pb.clone();
                    move |received, total| {
                        if pb.length() != Some(total) {
                            pb.set_length(total);
                        }
                        pb.set_position(received);
                    }
                };
                let mut result = client
                    .download_rom_with_cancel(
                        rom_id,
                        &save_path,
                        |_, _| interrupt.is_cancelled(),
                        &mut progress,
                    )
                    .await
                    .map(|_| {
                        if !pb.is_hidden() {
                            pb.finish_with_message(format!("✓ {name}"));
                        }
                    });

                if result.is_ok() && extract {
                    let extract_dir =
                        extraction_target_dir(&console_dir, &platform_slug, &stem, extract_layout);
                    if let Err(err) = tokio::fs::create_dir_all(&extract_dir).await {
                        result = Err(DownloadError::IoContext {
                            context: format!(
                                "failed to create extraction directory {extract_dir:?}"
                            ),
                            source: err,
                        });
                    } else if let Err(err) = extract_zip_archive(&save_path, &extract_dir) {
                        result = Err(err);
                    } else if delete_zip_after_extract {
                        if let Err(err) = tokio::fs::remove_file(&save_path).await {
                            result = Err(DownloadError::IoContext {
                                context: format!(
                                    "failed to delete zip {save_path:?} after extraction"
                                ),
                                source: err,
                            });
                        }
                    }
                }

                drop(permit);
                if let Err(e) = &result {
                    if !is_cancelled_download(e) {
                        eprintln!("error downloading {name} (id={rom_id}): {e}");
                    }
                }
                result.map(|_| save_path)
            }));
        }

        let mut summary = DownloadJsonSummary::default();
        for handle in handles {
            let task_result = tokio::select! {
                res = handle => res,
                _ = interrupt.cancelled() => {
                    summary.cancelled += 1;
                    continue;
                }
            };
            match task_result {
                Ok(Ok(path)) => {
                    summary.succeeded += 1;
                    summary.paths.push(path.display().to_string());
                }
                Ok(Err(e)) if is_cancelled_download(&e) => summary.cancelled += 1,
                _ => summary.failed += 1,
            }
        }

        if interrupt.is_cancelled() {
            presentation.emit_status("Interrupted by user.");
        }
        emit_download_summary(presentation, summary)
    } else {
        // ── Single ROM mode ────────────────────────────────────────────
        let rom_id = cmd.rom_id.ok_or_else(|| {
            RommError::Other(
                "ROM ID is required (e.g. 'download 123' or 'download batch --search-term ...')"
                    .into(),
            )
        })?;
        let rom = client.call(&GetRom { id: rom_id }).await?;
        let base_targets = build_base_rom_file_targets(&rom, &layout, &output_dir)?;

        let mut summary = DownloadJsonSummary::default();

        if !base_targets.is_empty() {
            summary = run_targets(base_targets, client, interrupt.clone(), 1, presentation).await?;
            if summary.failed > 0 || summary.cancelled > 0 || summary.succeeded == 0 {
                return Err(RommError::Other(
                    "base game download failed; not prompting for updates/DLC".into(),
                ));
            }
            presentation.emit_status("Base game files downloaded.");
        } else {
            let console_dir = resolve_console_roms_dir(&layout, &output_dir, &rom)?;
            tokio::fs::create_dir_all(&console_dir).await.map_err(|e| {
                RommError::Download(DownloadError::IoContext {
                    context: format!("create console download dir {console_dir:?}"),
                    source: e,
                })
            })?;
            let save_path = console_dir.join(format!("rom_{rom_id}.zip"));
            let style =
                presentation.progress_style(DOWNLOAD_PROGRESS_PLAIN, DOWNLOAD_PROGRESS_COLOR);
            let pb = if let Some(ref mp) = presentation.multi_progress() {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(style);
                pb
            } else {
                ProgressBar::hidden()
            };
            if interrupt.is_cancelled() {
                return Err(cancelled_download_error().into());
            }
            download_one(client, rom_id, &format!("ROM {rom_id}"), &save_path, pb).await?;
            summary.succeeded = 1;
            summary.paths.push(save_path.display().to_string());
            presentation.emit_status(format!("Saved to {save_path:?}"));
        }

        let extras_targets =
            build_update_dlc_targets_for_rom(client, &rom, &layout, &output_dir).await?;
        if !extras_targets.is_empty() {
            let include_extras = resolve_include_extras_choice(&cmd)?;
            if include_extras {
                let extras_summary =
                    run_targets(extras_targets, client, interrupt, cmd.jobs, presentation).await?;
                summary.succeeded += extras_summary.succeeded;
                summary.failed += extras_summary.failed;
                summary.cancelled += extras_summary.cancelled;
                summary.paths.extend(extras_summary.paths);
            }
        }

        emit_download_summary(presentation, summary)
    }
}

async fn handle_extras(
    cmd: DownloadExtrasCommand,
    client: &RommClient,
    interrupt: InterruptContext,
    layout: &RomsLayoutConfig,
    output_dir: PathBuf,
    jobs: usize,
    presentation: CliPresentation,
) -> Result<DownloadJsonSummary, RommError> {
    let targets = build_extras_targets(client, cmd.rom_id, layout, &output_dir).await?;
    run_targets(targets, client, interrupt, jobs, presentation).await
}

async fn run_targets(
    targets: Vec<DownloadTarget>,
    client: &RommClient,
    interrupt: InterruptContext,
    jobs: usize,
    presentation: CliPresentation,
) -> Result<DownloadJsonSummary, RommError> {
    if targets.is_empty() {
        presentation.emit_status("No downloadable extras were found.");
        return Ok(DownloadJsonSummary::default());
    }

    presentation.emit_status(format!(
        "Found {} download(s). Starting download with {} concurrent connections...",
        targets.len(),
        jobs
    ));

    let style = presentation.progress_style(DOWNLOAD_PROGRESS_PLAIN, DOWNLOAD_PROGRESS_COLOR);
    let mp = presentation.multi_progress();
    let semaphore = Arc::new(Semaphore::new(jobs));
    let mut handles = Vec::new();

    'enqueue: for target in targets {
        if interrupt.is_cancelled() {
            break 'enqueue;
        }
        let permit = semaphore.clone().acquire_owned().await.map_err(|_| {
            RommError::Other("download worker semaphore closed unexpectedly".into())
        })?;
        let client = client.clone();
        let interrupt = interrupt.clone();
        let destination = target.destination.clone();
        let title = target.title.clone();
        let kind = target.kind;
        let pb = if let Some(ref mp) = mp {
            let pb = mp.add(ProgressBar::new(0));
            pb.set_style(style.clone());
            pb
        } else {
            ProgressBar::hidden()
        };

        handles.push(tokio::spawn(async move {
            let result = download_target(&client, &target, &interrupt, pb).await;
            drop(permit);
            if let Err(err) = &result {
                if !is_cancelled_error(err) {
                    eprintln!(
                        "error downloading {title} ({kind:?}): {}",
                        format_command_error(err)
                    );
                }
            }
            result.map(|_| destination)
        }));
    }

    let mut summary = DownloadJsonSummary::default();
    for handle in handles {
        let task_result = tokio::select! {
            res = handle => res,
            _ = interrupt.cancelled() => {
                summary.cancelled += 1;
                continue;
            }
        };
        match task_result {
            Ok(Ok(path)) => {
                summary.succeeded += 1;
                summary.paths.push(path.display().to_string());
            }
            Ok(Err(e)) if is_cancelled_error(&e) => summary.cancelled += 1,
            _ => summary.failed += 1,
        }
    }

    if interrupt.is_cancelled() {
        presentation.emit_status("Interrupted by user.");
    }

    Ok(summary)
}

fn resolve_include_extras_choice(cmd: &DownloadCommand) -> Result<bool, RommError> {
    if cmd.with_extras || cmd.yes {
        return Ok(true);
    }
    if cmd.no_extras {
        return Ok(false);
    }
    if !is_interactive_terminal() {
        return Ok(false);
    }
    Confirm::new()
        .with_prompt("Updates/DLC are available. Download them now as extras?")
        .default(false)
        .interact()
        .map_err(|e| RommError::Other(format!("extras prompt failed: {e}")))
}

fn is_interactive_terminal() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn extraction_target_dir(
    output_dir: &std::path::Path,
    platform_slug: &str,
    rom_stem: &str,
    layout: ExtractLayout,
) -> PathBuf {
    let platform = utils::sanitize_filename(platform_slug);
    let rom = utils::sanitize_filename(rom_stem);
    match layout {
        ExtractLayout::Platform => output_dir.join(platform),
        ExtractLayout::Flat => output_dir.to_path_buf(),
        ExtractLayout::Rom => output_dir.join(platform).join(rom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use romm_api::core::resolve::resolve_platform_id_from_list;

    use crate::commands::{Cli, Commands};
    use romm_api::types::{Firmware, Platform};

    #[test]
    fn parse_download_batch_with_extract_flags() {
        let cli = Cli::parse_from([
            "romm-cli",
            "download",
            "batch",
            "--search-term",
            "Super Mario",
            "--extract",
            "--extract-layout",
            "platform",
            "--delete-zip-after-extract",
            "--jobs",
            "8",
        ]);

        let Commands::Download(cmd) = cli.command else {
            panic!("expected download command");
        };

        assert!(matches!(cmd.action, Some(DownloadAction::Batch)));
        assert_eq!(cmd.search_term.as_deref(), Some("Super Mario"));
        assert!(cmd.extract);
        assert_eq!(cmd.extract_layout, ExtractLayout::Platform);
        assert!(cmd.delete_zip_after_extract);
        assert_eq!(cmd.jobs, 8);
    }

    #[test]
    fn parse_download_batch_extract_defaults() {
        let cli = Cli::parse_from(["romm-cli", "download", "batch", "--search-term", "Metroid"]);

        let Commands::Download(cmd) = cli.command else {
            panic!("expected download command");
        };

        assert!(matches!(cmd.action, Some(DownloadAction::Batch)));
        assert!(!cmd.extract);
        assert_eq!(cmd.extract_layout, ExtractLayout::Platform);
        assert!(!cmd.delete_zip_after_extract);
    }

    #[test]
    fn parse_download_batch_with_platform_alias() {
        let cli = Cli::parse_from([
            "romm-cli",
            "download",
            "batch",
            "--platform",
            "3ds",
            "--search-term",
            "Mario",
        ]);

        let Commands::Download(cmd) = cli.command else {
            panic!("expected download command");
        };

        assert_eq!(cmd.platform.as_deref(), Some("3ds"));
    }

    #[test]
    fn parse_download_extras_command() {
        let cli = Cli::parse_from(["romm-cli", "download", "extras", "42"]);

        let Commands::Download(cmd) = cli.command else {
            panic!("expected download command");
        };

        let Some(DownloadAction::Extras(extras)) = cmd.action else {
            panic!("expected download extras");
        };
        assert_eq!(extras.rom_id, 42);
    }

    #[test]
    fn parse_download_batch_rejects_platform_id_flag() {
        let parsed = Cli::try_parse_from([
            "romm-cli",
            "download",
            "batch",
            "--platform",
            "3ds",
            "--platform-id",
            "3",
        ]);
        assert!(parsed.is_err(), "expected clap parse failure");
    }

    #[test]
    fn parse_download_rejects_zero_jobs() {
        let parsed = Cli::try_parse_from(["romm-cli", "download", "42", "--jobs", "0"]);
        assert!(parsed.is_err(), "expected --jobs 0 to fail");
    }

    #[test]
    fn parse_download_single_with_extras_flags() {
        let cli = Cli::parse_from(["romm-cli", "download", "42", "--with-extras", "--yes"]);
        let Commands::Download(cmd) = cli.command else {
            panic!("expected download command");
        };
        assert_eq!(cmd.rom_id, Some(42));
        assert!(cmd.with_extras);
        assert!(cmd.yes);
        assert!(!cmd.no_extras);
    }

    #[test]
    fn extraction_target_dir_platform_layout() {
        let dir = PathBuf::from("/tmp/out");
        let target = extraction_target_dir(
            &dir,
            "Nintendo Switch",
            "Mario (USA)",
            ExtractLayout::Platform,
        );
        assert_eq!(target, PathBuf::from("/tmp/out/Nintendo Switch"));
    }

    #[test]
    fn extraction_target_dir_rom_layout() {
        let dir = PathBuf::from("/tmp/out");
        let target = extraction_target_dir(&dir, "SNES", "Super Mario World", ExtractLayout::Rom);
        assert_eq!(target, PathBuf::from("/tmp/out/SNES/Super Mario World"));
    }

    #[test]
    fn resolve_platform_query_matches_slug_first() {
        let platforms = vec![platform_fixture(
            3,
            "3ds",
            "3ds",
            "Nintendo 3DS",
            None,
            None,
        )];
        let id = resolve_platform_id_from_list("3ds", &platforms).expect("slug should resolve");
        assert_eq!(id, 3);
    }

    #[test]
    fn resolve_platform_query_matches_name_case_insensitive() {
        let platforms = vec![platform_fixture(
            4,
            "nintendo-3ds",
            "3ds",
            "Nintendo 3DS",
            None,
            None,
        )];
        let id =
            resolve_platform_id_from_list("nintendo 3ds", &platforms).expect("name should resolve");
        assert_eq!(id, 4);
    }

    #[test]
    fn resolve_platform_query_errors_when_ambiguous() {
        let platforms = vec![
            platform_fixture(7, "foo-a", "foo-a", "Arcade", None, None),
            platform_fixture(8, "foo-b", "foo-b", "Arcade", None, None),
        ];
        let err =
            resolve_platform_id_from_list("Arcade", &platforms).expect_err("should be ambiguous");
        assert!(
            err.to_string().contains("ambiguous"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn resolve_platform_query_errors_when_missing() {
        let platforms = vec![platform_fixture(
            2,
            "gba",
            "gba",
            "Game Boy Advance",
            None,
            None,
        )];
        let err = resolve_platform_id_from_list("3ds", &platforms).expect_err("should not match");
        assert!(
            err.to_string().contains("No platform found"),
            "unexpected error: {err:#}"
        );
    }

    fn platform_fixture(
        id: u64,
        slug: &str,
        fs_slug: &str,
        name: &str,
        display_name: Option<&str>,
        custom_name: Option<&str>,
    ) -> Platform {
        Platform {
            id,
            slug: slug.to_string(),
            fs_slug: fs_slug.to_string(),
            rom_count: 0,
            name: name.to_string(),
            igdb_slug: None,
            moby_slug: None,
            hltb_slug: None,
            custom_name: custom_name.map(ToString::to_string),
            igdb_id: None,
            sgdb_id: None,
            moby_id: None,
            launchbox_id: None,
            ss_id: None,
            ra_id: None,
            hasheous_id: None,
            tgdb_id: None,
            flashpoint_id: None,
            category: None,
            generation: None,
            family_name: None,
            family_slug: None,
            url: None,
            url_logo: None,
            firmware: Vec::<Firmware>::new(),
            aspect_ratio: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            fs_size_bytes: 0,
            is_unidentified: false,
            is_identified: true,
            missing_from_fs: false,
            display_name: display_name.map(ToString::to_string),
        }
    }
}
