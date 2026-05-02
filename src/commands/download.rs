use anyhow::{anyhow, Result};
use clap::{Args, Subcommand, ValueEnum};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::client::RommClient;
use crate::core::download::{download_directory, extract_zip_archive, unique_zip_path};
use crate::core::interrupt::{cancelled_error, is_cancelled_error, InterruptContext};
use crate::core::utils;
use crate::endpoints::roms::GetRoms;
use crate::services::{PlatformService, RomService};
use crate::types::{Platform, Rom};

/// Maximum number of concurrent download connections.
const DEFAULT_CONCURRENCY: usize = 4;

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
    #[arg(long, default_value_t = DEFAULT_CONCURRENCY, global = true)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadAssetKind {
    RomArchive,
    Cover,
    Manual,
}

impl DownloadAssetKind {
    fn folder_name(self) -> &'static str {
        match self {
            DownloadAssetKind::RomArchive => "roms",
            DownloadAssetKind::Cover => "covers",
            DownloadAssetKind::Manual => "manuals",
        }
    }

    fn label(self) -> &'static str {
        match self {
            DownloadAssetKind::RomArchive => "ROM archive",
            DownloadAssetKind::Cover => "cover",
            DownloadAssetKind::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone)]
struct DownloadTarget {
    kind: DownloadAssetKind,
    title: String,
    source_url: String,
    source_query: Vec<(String, String)>,
    destination: PathBuf,
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
    .unwrap()
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

    client
        .download_url_with_query_with_cancel(
            &target.source_url,
            &target.source_query,
            &target.destination,
            |_, _| interrupt.is_cancelled(),
            &mut progress,
        )
        .await?;

    pb.finish_with_message(format!("✓ {}: {}", target.kind.label(), target.title));
    Ok(())
}

pub async fn handle(
    cmd: DownloadCommand,
    client: &RommClient,
    interrupt: Option<InterruptContext>,
) -> Result<()> {
    let interrupt = interrupt.unwrap_or_default();
    let output_dir = cmd.output.unwrap_or_else(download_directory);
    let action = cmd.action.clone();

    // Ensure output directory exists.
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| anyhow!("create download dir {:?}: {e}", output_dir))?;

    if let Some(DownloadAction::Extras(extras)) = action.clone() {
        return handle_extras(extras, client, interrupt, output_dir, cmd.jobs).await;
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

        let service = RomService::new(client);
        let results = service.search_roms(&ep).await?;

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
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = client.clone();
            let dir = output_dir.clone();
            let interrupt = interrupt.clone();
            let pb = mp.add(ProgressBar::new(0));
            pb.set_style(make_progress_style());

            let name = rom.name.clone();
            let rom_id = rom.id;
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
            let save_path = unique_zip_path(&dir, &stem);
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
                        extraction_target_dir(&dir, &platform_slug, &stem, extract_layout);
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

        let save_path = output_dir.join(format!("rom_{rom_id}.zip"));

        let mp = MultiProgress::new();
        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(make_progress_style());

        if interrupt.is_cancelled() {
            return Err(cancelled_error());
        }
        download_one(client, rom_id, &format!("ROM {rom_id}"), &save_path, pb).await?;

        println!("Saved to {:?}", save_path);
    }

    Ok(())
}

async fn handle_extras(
    cmd: DownloadExtrasCommand,
    client: &RommClient,
    interrupt: InterruptContext,
    output_dir: PathBuf,
    jobs: usize,
) -> Result<()> {
    let targets = build_extras_targets(client, cmd.rom_id, &output_dir).await?;

    if targets.is_empty() {
        println!("No downloadable extras were found for ROM {}.", cmd.rom_id);
        return Ok(());
    }

    println!(
        "Found {} extra download(s). Starting download with {} concurrent connections...",
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
        let permit = semaphore.clone().acquire_owned().await.unwrap();
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
        "\nExtras complete: {successes} succeeded, {failures} failed, {cancelled} cancelled."
    );

    Ok(())
}

async fn build_extras_targets(
    client: &RommClient,
    rom_id: u64,
    output_dir: &std::path::Path,
) -> Result<Vec<DownloadTarget>> {
    let service = RomService::new(client);
    let rom = service.get_rom(rom_id).await?;
    let extras_root = extras_root_dir(output_dir, &rom);

    let mut targets = Vec::new();
    targets.extend(build_related_rom_targets(client, &rom, &extras_root).await?);
    if let Some(cover) = build_cover_target(&rom, &extras_root) {
        targets.push(cover);
    }
    if let Some(manual) = build_manual_target(&rom, &extras_root) {
        targets.push(manual);
    }

    Ok(targets)
}

async fn build_related_rom_targets(
    client: &RommClient,
    rom: &Rom,
    extras_root: &std::path::Path,
) -> Result<Vec<DownloadTarget>> {
    let service = RomService::new(client);
    let ep = GetRoms {
        search_term: Some(rom.name.clone()),
        platform_id: Some(rom.platform_id),
        limit: Some(9999),
        ..Default::default()
    };
    let results = service.search_roms(&ep).await?;
    let groups = utils::group_roms_by_name(&results.items);
    let Some(group) = groups.iter().find(|g| g.name == rom.name) else {
        return Ok(Vec::new());
    };

    let mut targets = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut push_rom = |candidate: &Rom| {
        if candidate.id == rom.id || !seen.insert(candidate.id) {
            return;
        }
        let name = sanitize_extra_file_name(&candidate.fs_name);
        targets.push(DownloadTarget {
            kind: DownloadAssetKind::RomArchive,
            title: candidate.fs_name.clone(),
            source_url: "/api/roms/download".to_string(),
            source_query: vec![
                ("rom_ids".into(), candidate.id.to_string()),
                ("filename".into(), name.clone()),
            ],
            destination: extras_root
                .join(DownloadAssetKind::RomArchive.folder_name())
                .join(name),
        });
    };

    push_rom(&group.primary);
    for other in &group.others {
        push_rom(other);
    }
    Ok(targets)
}

fn build_cover_target(rom: &Rom, extras_root: &std::path::Path) -> Option<DownloadTarget> {
    let url = rom
        .url_cover
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())?;
    let filename = filename_from_url(url, "cover");
    Some(DownloadTarget {
        kind: DownloadAssetKind::Cover,
        title: rom.name.clone(),
        source_url: url.to_string(),
        source_query: Vec::new(),
        destination: extras_root
            .join(DownloadAssetKind::Cover.folder_name())
            .join(filename),
    })
}

fn build_manual_target(rom: &Rom, extras_root: &std::path::Path) -> Option<DownloadTarget> {
    let url = rom
        .url_manual
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())?;
    let filename = filename_from_url(url, "manual");
    Some(DownloadTarget {
        kind: DownloadAssetKind::Manual,
        title: rom.name.clone(),
        source_url: url.to_string(),
        source_query: Vec::new(),
        destination: extras_root
            .join(DownloadAssetKind::Manual.folder_name())
            .join(filename),
    })
}

fn extras_root_dir(output_dir: &std::path::Path, rom: &Rom) -> PathBuf {
    let platform_slug = rom
        .platform_fs_slug
        .clone()
        .or_else(|| rom.platform_slug.clone())
        .unwrap_or_else(|| format!("platform-{}", rom.platform_id));
    let game_slug = sanitized_extra_game_name(&rom.name, rom.id);
    output_dir.join(utils::sanitize_filename(&platform_slug)).join(game_slug).join("extras")
}

fn sanitized_extra_game_name(name: &str, rom_id: u64) -> String {
    let sanitized = utils::sanitize_filename(name);
    if sanitized.trim().is_empty() {
        format!("rom-{rom_id}")
    } else {
        sanitized
    }
}

fn sanitize_extra_file_name(name: &str) -> String {
    let sanitized = utils::sanitize_filename(name);
    if sanitized.trim().is_empty() {
        "download.bin".to_string()
    } else {
        sanitized
    }
}

fn filename_from_url(url: &str, fallback: &str) -> String {
    let fallback = sanitize_extra_file_name(fallback);
    reqwest::Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next_back().map(str::to_string))
        })
        .map(|name| sanitize_extra_file_name(&name))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(fallback)
}

async fn resolve_platform_id(
    client: &RommClient,
    platform_query: Option<&str>,
) -> Result<Option<u64>> {
    let Some(query) = platform_query.map(str::trim).filter(|q| !q.is_empty()) else {
        return Ok(None);
    };
    let service = PlatformService::new(client);
    let platforms = service.list_platforms().await?;
    resolve_platform_query(query, &platforms).map(Some)
}

fn resolve_platform_query(query: &str, platforms: &[Platform]) -> Result<u64> {
    let normalized = query.trim().to_ascii_lowercase();

    if let Some(platform) = platforms.iter().find(|p| {
        p.slug.eq_ignore_ascii_case(&normalized) || p.fs_slug.eq_ignore_ascii_case(&normalized)
    }) {
        return Ok(platform.id);
    }

    let exact_name_matches: Vec<&Platform> = platforms
        .iter()
        .filter(|p| {
            p.name.eq_ignore_ascii_case(&normalized)
                || p.display_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(&normalized))
                || p.custom_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(&normalized))
        })
        .collect();

    match exact_name_matches.len() {
        1 => Ok(exact_name_matches[0].id),
        0 => Err(anyhow!(
            "No platform found for '{}'. Use 'romm-cli platforms list' to inspect available values.",
            query
        )),
        _ => {
            let names = exact_name_matches
                .iter()
                .map(|p| format!("{} ({})", p.name, p.id))
                .collect::<Vec<_>>()
                .join(", ");
            Err(anyhow!(
                "Platform '{}' is ambiguous. Matches: {}. Please use a more specific --platform value.",
                query,
                names
            ))
        }
    }
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

    use crate::commands::{Cli, Commands};
    use crate::types::Firmware;

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
    fn extras_root_dir_is_sanitized() {
        let rom = rom_fixture(7, "Mario Kart", "Mario Kart [USA].zip");
        let dir = extras_root_dir(PathBuf::from("/tmp/out").as_path(), &rom);
        assert_eq!(
            dir,
            PathBuf::from("/tmp/out").join("Nintendo Switch").join("Mario Kart").join("extras")
        );
    }

    #[test]
    fn filename_from_url_uses_remote_leaf_or_fallback() {
        assert_eq!(
            filename_from_url("https://example.com/files/guide.pdf?download=1", "manual"),
            "guide.pdf"
        );
        assert_eq!(filename_from_url("not-a-url", "manual"), "manual");
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
        let id = resolve_platform_query("3ds", &platforms).expect("slug should resolve");
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
        let id = resolve_platform_query("nintendo 3ds", &platforms).expect("name should resolve");
        assert_eq!(id, 4);
    }

    #[test]
    fn resolve_platform_query_errors_when_ambiguous() {
        let platforms = vec![
            platform_fixture(7, "foo-a", "foo-a", "Arcade", None, None),
            platform_fixture(8, "foo-b", "foo-b", "Arcade", None, None),
        ];
        let err = resolve_platform_query("Arcade", &platforms).expect_err("should be ambiguous");
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
        let err = resolve_platform_query("3ds", &platforms).expect_err("should not match");
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

    fn rom_fixture(id: u64, name: &str, fs_name: &str) -> Rom {
        Rom {
            id,
            platform_id: 1,
            platform_slug: Some("switch".to_string()),
            platform_fs_slug: Some("Nintendo Switch".to_string()),
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: fs_name.to_string(),
            fs_name_no_tags: name.to_string(),
            fs_name_no_ext: name.to_string(),
            fs_extension: "zip".to_string(),
            fs_path: format!("/{id}.zip"),
            fs_size_bytes: 1,
            name: name.to_string(),
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
        }
    }
}
