use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use crate::client::RommClient;
use crate::commands::library_scan::{
    run_scan_library_flow, ScanCacheInvalidate, ScanLibraryOptions,
};
use crate::commands::print::print_roms_table;
use crate::commands::OutputFormat;
use crate::endpoints::roms::GetRoms;
use crate::services::{PlatformService, RomService};
use crate::types::Platform;

/// CLI entrypoint for listing/searching ROMs via `/api/roms`.
#[derive(Args, Debug)]
pub struct RomsCommand {
    /// Output as JSON (overrides global --json when set).
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub action: Option<RomsAction>,
}

#[derive(Subcommand, Debug)]
pub enum RomsAction {
    /// List available ROMs (default)
    #[command(visible_alias = "ls")]
    List {
        /// Search term to filter roms
        #[arg(long, visible_aliases = ["query", "q"])]
        search_term: Option<String>,
        /// Filter by platform slug or title (e.g. "3ds")
        #[arg(long)]
        platform: Option<String>,
        /// Page size limit
        #[arg(long)]
        limit: Option<u32>,
        /// Page offset
        #[arg(long)]
        offset: Option<u32>,
    },
    /// Get detailed information for a single ROM
    #[command(visible_alias = "info")]
    Get {
        /// The ID of the ROM
        id: u64,
    },
    /// Upload a ROM file to a platform
    #[command(visible_alias = "up")]
    Upload {
        /// Platform slug or name (e.g. "3ds", "Nintendo 3DS")
        #[arg(long)]
        platform: String,
        /// File or directory to upload
        file: PathBuf,
        /// Trigger a library scan after upload completes
        #[arg(short, long)]
        scan: bool,
        /// Wait until the library scan finishes (requires `--scan`; polls every 2 seconds)
        #[arg(long, requires = "scan")]
        wait: bool,
        /// Max seconds to wait when `--wait` is set (default: 3600)
        #[arg(long, requires = "wait")]
        wait_timeout_secs: Option<u64>,
    },
}

fn make_progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta}) {msg}",
    )
    .unwrap()
    .progress_chars("#>-")
}

async fn upload_one(
    client: &RommClient,
    platform_id: u64,
    file_path: std::path::PathBuf,
    pb: ProgressBar,
) -> Result<()> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    pb.set_message(format!("Uploading {}", filename));

    client
        .upload_rom(platform_id, &file_path, {
            let pb = pb.clone();
            move |uploaded, total| {
                if pb.length() != Some(total) {
                    pb.set_length(total);
                }
                pb.set_position(uploaded);
            }
        })
        .await?;

    pb.finish_with_message(format!("✓ Upload complete: {}", filename));
    Ok(())
}

pub async fn handle(cmd: RomsCommand, client: &RommClient, format: OutputFormat) -> Result<()> {
    let action = cmd.action.unwrap_or(RomsAction::List {
        search_term: None,
        platform: None,
        limit: None,
        offset: None,
    });

    match action {
        RomsAction::List {
            search_term,
            platform,
            limit,
            offset,
        } => {
            let resolved_platform_id = resolve_platform_id(client, platform.as_deref()).await?;
            let ep = GetRoms {
                search_term,
                platform_id: resolved_platform_id,
                collection_id: None,
                smart_collection_id: None,
                virtual_collection_id: None,
                limit,
                offset,
            };

            let service = RomService::new(client);
            let results = service.search_roms(&ep).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                }
                OutputFormat::Text => {
                    print_roms_table(&results);
                }
            }
        }
        RomsAction::Get { id } => {
            let service = RomService::new(client);
            let rom = service.get_rom(id).await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&rom)?);
                }
                OutputFormat::Text => {
                    // For now, reuse the table printer for a single item
                    // or just pretty-print the JSON.
                    println!("{}", serde_json::to_string_pretty(&rom)?);
                }
            }
        }
        RomsAction::Upload {
            file,
            platform,
            scan,
            wait,
            wait_timeout_secs,
        } => {
            let resolved_platform_id = match resolve_platform_id(client, Some(platform.trim())).await? {
                Some(id) => id,
                None => {
                    return Err(anyhow!(
                        "`--platform` must not be empty (use a slug or name from `romm-cli platforms list`)"
                    ));
                }
            };

            if !file.exists() {
                anyhow::bail!("File or directory does not exist: {:?}", file);
            }

            let mut files = Vec::new();
            if file.is_dir() {
                let mut entries = tokio::fs::read_dir(&file).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_file() {
                        files.push(path);
                    }
                }
                files.sort(); // Consistent order
            } else {
                files.push(file);
            }

            if files.is_empty() {
                println!("No files found to upload.");
                return Ok(());
            }

            if files.len() > 1 {
                println!("Found {} files to upload.", files.len());
            }

            let mp = indicatif::MultiProgress::new();
            let mut successes = 0u32;
            for path in files {
                let pb = mp.add(ProgressBar::new(0));
                pb.set_style(make_progress_style());
                match upload_one(client, resolved_platform_id, path.clone(), pb).await {
                    Ok(()) => successes += 1,
                    Err(e) => eprintln!("Error uploading {:?}: {}", path, e),
                }
            }

            if scan {
                if successes == 0 {
                    eprintln!("Skipping library scan: no uploads completed successfully.");
                } else {
                    let options = ScanLibraryOptions {
                        wait,
                        wait_timeout: Duration::from_secs(wait_timeout_secs.unwrap_or(3600)),
                        cache_invalidate: if wait {
                            ScanCacheInvalidate::Platform(resolved_platform_id)
                        } else {
                            ScanCacheInvalidate::None
                        },
                    };
                    run_scan_library_flow(client, options, format, None).await?;
                }
            }
        }
    }

    Ok(())
}

async fn resolve_platform_id(client: &RommClient, platform_query: Option<&str>) -> Result<Option<u64>> {
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
        0 => anyhow::bail!(
            "No platform found for '{}'. Use 'romm-cli platforms list' to inspect available values.",
            query
        ),
        _ => {
            let names = exact_name_matches
                .iter()
                .map(|p| format!("{} ({})", p.name, p.id))
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!(
                "Platform '{}' is ambiguous. Matches: {}. Please use a more specific --platform value.",
                query,
                names
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    use crate::commands::{Cli, Commands};

    #[test]
    fn parse_roms_list_with_platform_filter() {
        let cli = Cli::parse_from(["romm-cli", "roms", "list", "--platform", "3ds", "--limit", "10"]);
        let Commands::Roms(cmd) = cli.command else {
            panic!("expected roms command");
        };
        let Some(RomsAction::List {
            platform,
            limit,
            ..
        }) = cmd.action
        else {
            panic!("expected roms list");
        };
        assert_eq!(platform.as_deref(), Some("3ds"));
        assert_eq!(limit, Some(10));
    }

    #[test]
    fn parse_roms_get_rejects_list_only_filter() {
        let parsed = Cli::try_parse_from(["romm-cli", "roms", "get", "1", "--platform", "3ds"]);
        assert!(parsed.is_err(), "expected clap parse failure");
    }

    #[test]
    fn parse_roms_list_rejects_platform_id_flag() {
        let parsed = Cli::try_parse_from(["romm-cli", "roms", "list", "--platform-id", "3"]);
        assert!(parsed.is_err(), "expected clap parse failure");
    }

    #[test]
    fn parse_roms_upload_requires_platform() {
        let parsed = Cli::try_parse_from(["romm-cli", "roms", "upload", "foo.bin"]);
        assert!(parsed.is_err(), "expected clap parse failure without --platform");
    }

    #[test]
    fn parse_roms_upload_with_platform_and_file() {
        let cli = Cli::parse_from([
            "romm-cli",
            "roms",
            "upload",
            "--platform",
            "3ds",
            "foo.bin",
        ]);
        let Commands::Roms(cmd) = cli.command else {
            panic!("expected roms command");
        };
        let Some(RomsAction::Upload { platform, file, .. }) = cmd.action else {
            panic!("expected roms upload");
        };
        assert_eq!(platform, "3ds");
        assert_eq!(file, PathBuf::from("foo.bin"));
    }
}
