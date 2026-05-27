use anyhow::{anyhow, Result};
use clap::{Args, Subcommand, ValueEnum};
use dialoguer::Confirm;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::client::RommClient;
use crate::config::{load_config, RomsLayoutConfig};
use crate::core::download::{
    extract_zip_archive, prepare_download_target_destination, resolve_console_roms_dir,
    resolve_download_directory, unique_zip_path,
};
use crate::core::extras::{
    build_base_rom_file_targets, build_extras_targets, build_update_dlc_targets_for_rom,
    DownloadTarget,
};
use crate::core::interrupt::{cancelled_error, is_cancelled_error, InterruptContext};
use crate::core::utils;
use crate::endpoints::roms::{GetRom, GetRoms};
use crate::core::resolve::resolve_platform_id;
/// Maximum number of concurrent download connections.
const DEFAULT_CONCURRENCY: usize = 4;

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

fn make_progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}",
    )
    .expect("hardcoded download progress template")
    .progress_chars("#>-")
}

async fn download_one(
    client: &RommClient,
    rom_id: u64,
    name: &str,
    save_path: &std::path::Path,
    pb: ProgressBar,
) -> Result<()> {
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
) -> Result<()> {
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

    let urls = candidate_download_urls(target);
    let mut last_err: Option<anyhow::Error> = None;
    if prepare_download_target_destination(target).await? {
        if let Some(expected_size) = target.expected_size_bytes {
            progress(expected_size, expected_size);
        }
        pb.finish_with_message(format!("✓ {}: {}", target.kind.label(), target.title));
        return Ok(());
    }
    for url in urls {
        match client
            .download_url_with_query_with_cancel(
                &url,
                &target.source_query,
                &target.destination,
                |_, _| interrupt.is_cancelled(),
                &mut progress,
            )
            .await
        {
            Ok(()) => {
                last_err = None;
                break;
            }
            Err(err) => {
                if !err.to_string().contains("404 Not Found") {
                    return Err(err);
                }
                last_err = Some(err);
            }
        }
    }
    if let Some(err) = last_err {
        return Err(err);
    }

    pb.finish_with_message(format!("✓ {}: {}", target.kind.label(), target.title));
    Ok(())
}

fn candidate_download_urls(target: &DownloadTarget) -> Vec<String> {
    if target.kind != crate::core::extras::DownloadAssetKind::RomFile {
        return vec![target.source_url.clone()];
    }
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

pub async fn handle(
    cmd: DownloadCommand,
    client: &RommClient,
    interrupt: Option<InterruptContext>,
) -> Result<()> {
    let interrupt = interrupt.unwrap_or_default();
    let config = load_config()?;
    let layout = config.roms_layout.clone();
    let output_dir = match cmd.output.clone() {
        Some(path) => path,
        None => resolve_download_directory(Some(config.download_dir.as_str()))?,
    };
    let action = cmd.action.clone();

    if cmd.with_extras && cmd.no_extras {
        return Err(anyhow!(
            "--with-extras and --no-extras are mutually exclusive"
        ));
    }

    // Ensure output directory exists.
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| anyhow!("create download dir {:?}: {e}", output_dir))?;

    if let Some(DownloadAction::Extras(extras)) = action.clone() {
        return handle_extras(extras, client, interrupt, &layout, output_dir, cmd.jobs).await;
    }

    // Determine if we are in batch mode.
    let is_batch = matches!(action, Some(DownloadAction::Batch));

    if is_batch {
        // ── Batch mode ─────────────────────────────────────────────────
        if cmd.platform.is_none() && cmd.search_term.is_none() {
            return Err(anyhow!(
                "Batch download requires at least --platform or --search-term to scope the download"
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
            println!("No ROMs found matching the given filters.");
            return Ok(());
        }

        println!(
            "Found {} ROM(s). Starting download with {} concurrent connections...",
            results.items.len(),
            cmd.jobs
        );

        let mp = MultiProgress::new();
        let semaphore = Arc::new(Semaphore::new(cmd.jobs));
        let mut handles = Vec::new();

        'enqueue: for rom in results.items {
            if interrupt.is_cancelled() {
                break 'enqueue;
            }
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| anyhow!("download worker semaphore closed unexpectedly"))?;
            let client = client.clone();
            let base_dir = output_dir.clone();
            let layout = layout.clone();
            let interrupt = interrupt.clone();
            let pb = mp.add(ProgressBar::new(0));
            pb.set_style(make_progress_style());

            let name = rom.name.clone();
            let rom_id = rom.id;
            let console_dir = resolve_console_roms_dir(&layout, &base_dir, &rom)?;
            tokio::fs::create_dir_all(&console_dir)
                .await
                .map_err(|e| anyhow!("create console download dir {:?}: {e}", console_dir))?;
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
                        pb.finish_with_message(format!("✓ {name}"));
                    });

                if result.is_ok() && extract {
                    let extract_dir =
                        extraction_target_dir(&console_dir, &platform_slug, &stem, extract_layout);
                    if let Err(err) = tokio::fs::create_dir_all(&extract_dir).await {
                        result = Err(anyhow!(
                            "failed to create extraction directory {:?}: {}",
                            extract_dir,
                            err
                        ));
                    } else if let Err(err) = extract_zip_archive(&save_path, &extract_dir) {
                        result = Err(anyhow!(
                            "failed to extract {:?} to {:?}: {}",
                            save_path,
                            extract_dir,
                            err
                        ));
                    } else if delete_zip_after_extract {
                        tokio::fs::remove_file(&save_path).await.map_err(|err| {
                            anyhow!(
                                "failed to delete zip {:?} after extraction: {}",
                                save_path,
                                err
                            )
                        })?;
                    }
                }

                drop(permit);
                if let Err(e) = &result {
                    if !is_cancelled_error(e) {
                        eprintln!("error downloading {name} (id={rom_id}): {e}");
                    }
                }
                result
            }));
        }

        let mut successes = 0u32;
        let mut failures = 0u32;
        let mut cancelled = 0u32;
        for handle in handles {
            let task_result = tokio::select! {
                res = handle => res,
                _ = interrupt.cancelled() => {
                    cancelled += 1;
                    continue;
                }
            };
            match task_result {
                Ok(Ok(())) => successes += 1,
                Ok(Err(e)) if is_cancelled_error(&e) => cancelled += 1,
                _ => failures += 1,
            }
        }

        if interrupt.is_cancelled() {
            println!("\nInterrupted by user.");
        }
        println!(
            "\nBatch complete: {successes} succeeded, {failures} failed, {cancelled} cancelled."
        );
    } else {
        // ── Single ROM mode ────────────────────────────────────────────
        let rom_id = cmd.rom_id.ok_or_else(|| {
            anyhow!(
                "ROM ID is required (e.g. 'download 123' or 'download batch --search-term ...')"
            )
        })?;
        let rom = client.call(&GetRom { id: rom_id }).await?;
        let base_targets = build_base_rom_file_targets(&rom, &layout, &output_dir)?;

        if !base_targets.is_empty() {
            let summary = run_targets(base_targets, client, interrupt.clone(), 1).await?;
            if summary.failures > 0 || summary.cancelled > 0 || summary.successes == 0 {
                return Err(anyhow!(
                    "base game download failed; not prompting for updates/DLC"
                ));
            }
            println!("Base game files downloaded.");
        } else {
            let console_dir = resolve_console_roms_dir(&layout, &output_dir, &rom)?;
            tokio::fs::create_dir_all(&console_dir)
                .await
                .map_err(|e| anyhow!("create console download dir {:?}: {e}", console_dir))?;
            let save_path = console_dir.join(format!("rom_{rom_id}.zip"));
            let mp = MultiProgress::new();
            let pb = mp.add(ProgressBar::new(0));
            pb.set_style(make_progress_style());
            if interrupt.is_cancelled() {
                return Err(cancelled_error());
            }
            download_one(client, rom_id, &format!("ROM {rom_id}"), &save_path, pb).await?;
            println!("Saved to {:?}", save_path);
        }

        let extras_targets =
            build_update_dlc_targets_for_rom(client, &rom, &layout, &output_dir).await?;
        if !extras_targets.is_empty() {
            let include_extras = resolve_include_extras_choice(&cmd)?;
            if include_extras {
                run_targets(extras_targets, client, interrupt, cmd.jobs).await?;
            }
        }
    }

    Ok(())
}

async fn handle_extras(
    cmd: DownloadExtrasCommand,
    client: &RommClient,
    interrupt: InterruptContext,
    layout: &RomsLayoutConfig,
    output_dir: PathBuf,
    jobs: usize,
) -> Result<()> {
    let targets = build_extras_targets(client, cmd.rom_id, layout, &output_dir).await?;
    run_targets(targets, client, interrupt, jobs).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct DownloadRunSummary {
    successes: u32,
    failures: u32,
    cancelled: u32,
}

async fn run_targets(
    targets: Vec<DownloadTarget>,
    client: &RommClient,
    interrupt: InterruptContext,
    jobs: usize,
) -> Result<DownloadRunSummary> {
    if targets.is_empty() {
        println!("No downloadable extras were found.");
        return Ok(DownloadRunSummary {
            successes: 0,
            failures: 0,
            cancelled: 0,
        });
    }

    println!(
        "Found {} download(s). Starting download with {} concurrent connections...",
        targets.len(),
        jobs
    );

    let mp = MultiProgress::new();
    let semaphore = Arc::new(Semaphore::new(jobs));
    let mut handles = Vec::new();

    'enqueue: for target in targets {
        if interrupt.is_cancelled() {
            break 'enqueue;
        }
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| anyhow!("download worker semaphore closed unexpectedly"))?;
        let client = client.clone();
        let interrupt = interrupt.clone();
        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(make_progress_style());

        handles.push(tokio::spawn(async move {
            let result = download_target(&client, &target, &interrupt, pb).await;
            drop(permit);
            if let Err(err) = &result {
                if !is_cancelled_error(err) {
                    eprintln!(
                        "error downloading {} ({:?}): {}",
                        target.title, target.kind, err
                    );
                }
            }
            result
        }));
    }

    let mut successes = 0u32;
    let mut failures = 0u32;
    let mut cancelled = 0u32;
    for handle in handles {
        let task_result = tokio::select! {
            res = handle => res,
            _ = interrupt.cancelled() => {
                cancelled += 1;
                continue;
            }
        };
        match task_result {
            Ok(Ok(())) => successes += 1,
            Ok(Err(e)) if is_cancelled_error(&e) => cancelled += 1,
            _ => failures += 1,
        }
    }

    if interrupt.is_cancelled() {
        println!("\nInterrupted by user.");
    }
    println!(
        "\nDownload complete: {successes} succeeded, {failures} failed, {cancelled} cancelled."
    );

    Ok(DownloadRunSummary {
        successes,
        failures,
        cancelled,
    })
}

fn resolve_include_extras_choice(cmd: &DownloadCommand) -> Result<bool> {
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
        .map_err(|e| anyhow!("extras prompt failed: {e}"))
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
    use crate::core::resolve::resolve_platform_id_from_list;
    use clap::Parser;

    use crate::commands::{Cli, Commands};
    use crate::types::{Firmware, Platform};

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
    fn rom_file_download_candidates_use_official_romsfiles_endpoint() {
        let target = DownloadTarget {
            kind: crate::core::extras::DownloadAssetKind::RomFile,
            title: "DLC".into(),
            source_url: "/api/roms/12/files/content/dlc%2Ensp".into(),
            source_query: Vec::new(),
            destination: PathBuf::from("/tmp/dlc.nsp"),
            expected_size_bytes: Some(12),
        };

        assert_eq!(
            candidate_download_urls(&target),
            vec![
                "/api/roms/12/files/content/dlc%2Ensp".to_string(),
                "/api/romsfiles/12/content/dlc%2Ensp".to_string(),
                "/api/roms/files/12/content/dlc%2Ensp".to_string()
            ]
        );
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
        let id = resolve_platform_id_from_list("nintendo 3ds", &platforms).expect("name should resolve");
        assert_eq!(id, 4);
    }

    #[test]
    fn resolve_platform_query_errors_when_ambiguous() {
        let platforms = vec![
            platform_fixture(7, "foo-a", "foo-a", "Arcade", None, None),
            platform_fixture(8, "foo-b", "foo-b", "Arcade", None, None),
        ];
        let err = resolve_platform_id_from_list("Arcade", &platforms).expect_err("should be ambiguous");
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
