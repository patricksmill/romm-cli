use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;

use crate::client::RommClient;
use crate::commands::library_scan::{
    run_scan_library_flow, ScanCacheInvalidate, ScanLibraryOptions,
};
use crate::commands::print::print_roms_table;
use crate::commands::OutputFormat;
use crate::endpoints::roms::{
    DeleteRomNote, DeleteRoms, GetRomByHash, GetRomByMetadataProvider, GetRomFilters, GetRoms,
    GetRomNotes, GetSearchCover, GetSearchRoms, PostRomNote, PutRomNote, PutRomUserProps,
};
use crate::services::{
    resolve_manual_collection_id, resolve_platform_ids, resolve_smart_collection_id, RomService,
};
use crate::services::{self};

/// Optional tri-state: CLI passes `true` / `false` / `yes` / `no` / `1` / `0`.
fn parse_opt_bool(label: &str, raw: &Option<String>) -> Result<Option<bool>> {
    let Some(s) = raw else {
        return Ok(None);
    };
    let t = s.trim().to_ascii_lowercase();
    if t.is_empty() {
        return Ok(None);
    }
    match t.as_str() {
        "true" | "1" | "yes" | "y" => Ok(Some(true)),
        "false" | "0" | "no" | "n" => Ok(Some(false)),
        _ => Err(anyhow!(
            "Invalid boolean for {}: {:?} (use true or false)",
            label,
            s
        )),
    }
}

/// `roms list` flags (also used as default when no subcommand).
#[derive(Args, Debug, Clone, Default)]
pub struct RomListArgs {
    #[arg(long, visible_aliases = ["query", "q"])]
    pub search_term: Option<String>,
    /// Platform slug or name; repeat for multiple `platform_ids`
    #[arg(long, action = clap::ArgAction::Append, visible_alias = "p")]
    pub platform: Vec<String>,
    /// Manual collection id or exact name
    #[arg(long)]
    pub collection: Option<String>,
    /// Smart collection id or exact name
    #[arg(long)]
    pub smart_collection: Option<String>,
    /// Virtual collection id (e.g. recent)
    #[arg(long)]
    pub virtual_collection: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub offset: Option<u32>,
    #[arg(long)]
    pub matched: Option<String>,
    #[arg(long)]
    pub favorite: Option<String>,
    #[arg(long)]
    pub duplicate: Option<String>,
    #[arg(long)]
    pub last_played: Option<String>,
    #[arg(long)]
    pub playable: Option<String>,
    #[arg(long)]
    pub missing: Option<String>,
    #[arg(long)]
    pub has_ra: Option<String>,
    #[arg(long)]
    pub verified: Option<String>,
    #[arg(long)]
    pub group_by_meta_id: Option<String>,
    #[arg(long)]
    pub with_char_index: Option<String>,
    #[arg(long)]
    pub with_filter_values: Option<String>,
    #[arg(long = "genre", action = clap::ArgAction::Append)]
    pub genres: Vec<String>,
    #[arg(long = "franchise", action = clap::ArgAction::Append)]
    pub franchises: Vec<String>,
    #[arg(long = "collection-tag", action = clap::ArgAction::Append)]
    pub collection_tags: Vec<String>,
    #[arg(long = "company", action = clap::ArgAction::Append)]
    pub companies: Vec<String>,
    #[arg(long = "age-rating", action = clap::ArgAction::Append)]
    pub age_ratings: Vec<String>,
    #[arg(long = "status", action = clap::ArgAction::Append)]
    pub statuses: Vec<String>,
    #[arg(long = "region", action = clap::ArgAction::Append)]
    pub regions: Vec<String>,
    #[arg(long = "language", action = clap::ArgAction::Append)]
    pub languages: Vec<String>,
    #[arg(long = "player-count", action = clap::ArgAction::Append)]
    pub player_counts: Vec<String>,
    #[arg(long)]
    pub genres_logic: Option<String>,
    #[arg(long)]
    pub franchises_logic: Option<String>,
    #[arg(long)]
    pub collections_logic: Option<String>,
    #[arg(long)]
    pub companies_logic: Option<String>,
    #[arg(long)]
    pub age_ratings_logic: Option<String>,
    #[arg(long)]
    pub regions_logic: Option<String>,
    #[arg(long)]
    pub languages_logic: Option<String>,
    #[arg(long)]
    pub statuses_logic: Option<String>,
    #[arg(long)]
    pub player_counts_logic: Option<String>,
    #[arg(long)]
    pub order_by: Option<String>,
    #[arg(long)]
    pub order_dir: Option<String>,
    #[arg(long)]
    pub updated_after: Option<String>,
}

async fn build_get_roms(client: &RommClient, a: RomListArgs) -> Result<GetRoms> {
    let platform_ids = resolve_platform_ids(client, &a.platform).await?;
    let mut platform_id = None;
    let mut extra = platform_ids;
    if extra.len() == 1 {
        platform_id = Some(extra[0]);
        extra.clear();
    } else if extra.len() > 1 {
        platform_id = None;
    }

    Ok(GetRoms {
        search_term: a.search_term,
        platform_id,
        platform_ids: extra,
        collection_id: resolve_manual_collection_id(client, a.collection.as_deref()).await?,
        smart_collection_id: resolve_smart_collection_id(client, a.smart_collection.as_deref())
            .await?,
        virtual_collection_id: a
            .virtual_collection
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        matched: parse_opt_bool("matched", &a.matched)?,
        favorite: parse_opt_bool("favorite", &a.favorite)?,
        duplicate: parse_opt_bool("duplicate", &a.duplicate)?,
        last_played: parse_opt_bool("last_played", &a.last_played)?,
        playable: parse_opt_bool("playable", &a.playable)?,
        missing: parse_opt_bool("missing", &a.missing)?,
        has_ra: parse_opt_bool("has_ra", &a.has_ra)?,
        verified: parse_opt_bool("verified", &a.verified)?,
        group_by_meta_id: parse_opt_bool("group_by_meta_id", &a.group_by_meta_id)?,
        with_char_index: parse_opt_bool("with_char_index", &a.with_char_index)?,
        with_filter_values: parse_opt_bool("with_filter_values", &a.with_filter_values)?,
        genres: a.genres,
        franchises: a.franchises,
        collections: a.collection_tags,
        companies: a.companies,
        age_ratings: a.age_ratings,
        statuses: a.statuses,
        regions: a.regions,
        languages: a.languages,
        player_counts: a.player_counts,
        genres_logic: a.genres_logic,
        franchises_logic: a.franchises_logic,
        collections_logic: a.collections_logic,
        companies_logic: a.companies_logic,
        age_ratings_logic: a.age_ratings_logic,
        regions_logic: a.regions_logic,
        languages_logic: a.languages_logic,
        statuses_logic: a.statuses_logic,
        player_counts_logic: a.player_counts_logic,
        order_by: a.order_by,
        order_dir: a.order_dir,
        updated_after: a.updated_after,
        limit: a.limit,
        offset: a.offset,
    })
}

/// CLI entrypoint for listing/searching ROMs via `/api/roms`.
#[derive(Args, Debug)]
pub struct RomsCommand {
    /// Output as JSON (overrides global --json when set).
    #[arg(long, global = true)]
    pub json: bool,

    /// Flags for the default `list` action (`romm-cli roms` with no subcommand, or before a subcommand).
    #[command(flatten)]
    pub list: RomListArgs,

    #[command(subcommand)]
    pub action: Option<RomsAction>,
}

#[derive(Subcommand, Debug)]
pub enum RomsAction {
    /// Get detailed information for a single ROM
    #[command(visible_alias = "info")]
    Get {
        /// The ID of the ROM
        id: u64,
    },
    /// Lookup ROM by file hash or metadata provider id
    Find {
        #[arg(long)]
        crc: Option<String>,
        #[arg(long)]
        md5: Option<String>,
        #[arg(long)]
        sha1: Option<String>,
        #[arg(long)]
        igdb_id: Option<i64>,
        #[arg(long)]
        moby_id: Option<i64>,
        #[arg(long)]
        ss_id: Option<i64>,
        #[arg(long)]
        ra_id: Option<i64>,
        #[arg(long)]
        launchbox_id: Option<i64>,
        #[arg(long)]
        hasheous_id: Option<i64>,
        #[arg(long)]
        tgdb_id: Option<i64>,
        #[arg(long)]
        flashpoint_id: Option<String>,
        #[arg(long)]
        hltb_id: Option<i64>,
    },
    /// Print canonical filter values from `GET /api/roms/filters`
    Filters,
    /// Delete ROMs from the database (optional filesystem delete)
    Delete {
        /// ROM ids to remove from the database
        #[arg(required = true)]
        rom_ids: Vec<u64>,
        /// Also delete these ROM ids from disk (repeat ids as needed)
        #[arg(long, action = clap::ArgAction::Append)]
        delete_from_fs: Vec<u64>,
        /// Skip confirmation
        #[arg(long)]
        yes: bool,
    },
    /// Update per-user ROM properties (`PUT /api/roms/{id}/props`)
    Props {
        id: u64,
        #[arg(long)]
        is_main_sibling: Option<String>,
        #[arg(long)]
        backlogged: Option<String>,
        #[arg(long)]
        now_playing: Option<String>,
        #[arg(long)]
        hidden: Option<String>,
        #[arg(long)]
        rating: Option<u8>,
        #[arg(long)]
        difficulty: Option<u8>,
        #[arg(long)]
        completion: Option<u8>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        update_last_played: bool,
        #[arg(long)]
        remove_last_played: bool,
    },
    /// List notes for a ROM
    NotesList {
        rom_id: u64,
        #[arg(long)]
        public_only: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long = "tag", action = clap::ArgAction::Append)]
        tags: Vec<String>,
    },
    /// Add a note (JSON body string, e.g. {\"title\":\"t\",\"content\":\"c\"})
    NotesAdd {
        rom_id: u64,
        /// JSON object
        #[arg(long)]
        json: String,
    },
    /// Update a note
    NotesUpdate {
        rom_id: u64,
        note_id: u64,
        #[arg(long)]
        json: String,
    },
    /// Delete a note
    NotesDelete { rom_id: u64, note_id: u64 },
    /// Upload a manual file (`POST /api/roms/{id}/manuals`)
    ManualsAdd { rom_id: u64, file: PathBuf },
    /// Search covers and metadata matches
    CoverSearch {
        rom_id: u64,
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "name")]
        search_by: String,
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
    match cmd.action {
        None => {
            let ep = build_get_roms(client, cmd.list.clone()).await?;
            let service = RomService::new(client);
            let results = service.search_roms(&ep).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&results)?),
                OutputFormat::Text => print_roms_table(&results),
            }
        }
        Some(RomsAction::Get { id }) => {
            let service = RomService::new(client);
            let rom = service.get_rom(id).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rom)?),
                OutputFormat::Text => println!("{}", serde_json::to_string_pretty(&rom)?),
            }
        }
        Some(RomsAction::Find {
            crc,
            md5,
            sha1,
            igdb_id,
            moby_id,
            ss_id,
            ra_id,
            launchbox_id,
            hasheous_id,
            tgdb_id,
            flashpoint_id,
            hltb_id,
        }) => {
            let hash_ep = GetRomByHash {
                crc_hash: crc.clone(),
                md5_hash: md5.clone(),
                sha1_hash: sha1.clone(),
            };
            let has_hash = crc.is_some() || md5.is_some() || sha1.is_some();
            let has_meta = igdb_id.is_some()
                || moby_id.is_some()
                || ss_id.is_some()
                || ra_id.is_some()
                || launchbox_id.is_some()
                || hasheous_id.is_some()
                || tgdb_id.is_some()
                || flashpoint_id.is_some()
                || hltb_id.is_some();
            if has_hash == has_meta {
                anyhow::bail!("Specify either hash flags (--crc/--md5/--sha1) or metadata id flags (--igdb-id, ...), not both.");
            }
            let v = if has_hash {
                client.call(&hash_ep).await?
            } else {
                client
                    .call(&GetRomByMetadataProvider {
                        igdb_id,
                        moby_id,
                        ss_id,
                        ra_id,
                        launchbox_id,
                        hasheous_id,
                        tgdb_id,
                        flashpoint_id,
                        hltb_id,
                    })
                    .await?
            };
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::Filters) => {
            let v = client.call(&GetRomFilters).await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::Delete {
            rom_ids,
            delete_from_fs,
            yes,
        }) => {
            if !yes {
                let ok = Confirm::new()
                    .with_prompt(format!(
                        "Delete {} ROM(s) from the database (and {} from disk)?",
                        rom_ids.len(),
                        delete_from_fs.len()
                    ))
                    .interact()?;
                if !ok {
                    return Ok(());
                }
            }
            let v = client
                .call(&DeleteRoms {
                    roms: rom_ids,
                    delete_from_fs,
                })
                .await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::Props {
            id,
            is_main_sibling,
            backlogged,
            now_playing,
            hidden,
            rating,
            difficulty,
            completion,
            status,
            update_last_played,
            remove_last_played,
        }) => {
            if update_last_played && remove_last_played {
                anyhow::bail!("--update-last-played and --remove-last-played are mutually exclusive.");
            }
            let mut body = json!({});
            let obj = body.as_object_mut().unwrap();
            if let Some(b) = parse_opt_bool("is_main_sibling", &is_main_sibling)? {
                obj.insert("is_main_sibling".into(), json!(b));
            }
            if let Some(b) = parse_opt_bool("backlogged", &backlogged)? {
                obj.insert("backlogged".into(), json!(b));
            }
            if let Some(b) = parse_opt_bool("now_playing", &now_playing)? {
                obj.insert("now_playing".into(), json!(b));
            }
            if let Some(b) = parse_opt_bool("hidden", &hidden)? {
                obj.insert("hidden".into(), json!(b));
            }
            if let Some(r) = rating {
                obj.insert("rating".into(), json!(r));
            }
            if let Some(d) = difficulty {
                obj.insert("difficulty".into(), json!(d));
            }
            if let Some(c) = completion {
                obj.insert("completion".into(), json!(c));
            }
            if let Some(ref s) = status {
                if !s.is_empty() {
                    obj.insert("status".into(), json!(s));
                }
            }
            let v = client
                .call(&PutRomUserProps {
                    rom_id: id,
                    body,
                    update_last_played,
                    remove_last_played,
                })
                .await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::NotesList {
            rom_id,
            public_only,
            search,
            tags,
        }) => {
            let v = client
                .call(&GetRomNotes {
                    rom_id,
                    public_only: parse_opt_bool("public_only", &public_only)?,
                    search,
                    tags,
                })
                .await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::NotesAdd { rom_id, json: body }) => {
            let parsed: serde_json::Value = serde_json::from_str(&body)?;
            let v = client.call(&PostRomNote { rom_id, body: parsed }).await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::NotesUpdate {
            rom_id,
            note_id,
            json: body,
        }) => {
            let parsed: serde_json::Value = serde_json::from_str(&body)?;
            let v = client
                .call(&PutRomNote {
                    rom_id,
                    note_id,
                    body: parsed,
                })
                .await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::NotesDelete { rom_id, note_id }) => {
            let v = client.call(&DeleteRomNote { rom_id, note_id }).await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::ManualsAdd { rom_id, file }) => {
            let v = client.upload_rom_manual(rom_id, &file).await?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        Some(RomsAction::CoverSearch {
            rom_id,
            query,
            search_by,
        }) => {
            let cover = client
                .call(&GetSearchCover {
                    search_term: query.clone(),
                })
                .await?;
            let roms = client
                .call(&GetSearchRoms {
                    rom_id,
                    search_term: Some(query),
                    search_by: Some(search_by),
                })
                .await?;
            let out = json!({ "cover": cover, "roms": roms });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        Some(RomsAction::Upload {
            file,
            platform,
            scan,
            wait,
            wait_timeout_secs,
        }) => {
            let resolved_platform_id = match services::resolve_platform_id(client, Some(platform.trim())).await?
            {
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
                files.sort();
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
                        task_kwargs: None,
                    };
                    run_scan_library_flow(client, options, format, None).await?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    use crate::commands::{Cli, Commands};

    #[test]
    fn parse_roms_list_with_platform_filter() {
        let cli = Cli::parse_from([
            "romm-cli",
            "roms",
            "--platform",
            "3ds",
            "--limit",
            "10",
        ]);
        let Commands::Roms(cmd) = cli.command else {
            panic!("expected roms command");
        };
        assert!(cmd.action.is_none());
        assert_eq!(cmd.list.platform, vec!["3ds".to_string()]);
        assert_eq!(cmd.list.limit, Some(10));
    }

    #[test]
    fn parse_roms_get_rejects_list_only_filter() {
        let parsed = Cli::try_parse_from(["romm-cli", "roms", "get", "1", "--platform", "3ds"]);
        assert!(parsed.is_err(), "expected clap parse failure");
    }

    #[test]
    fn parse_roms_list_rejects_platform_id_flag() {
        let parsed = Cli::try_parse_from(["romm-cli", "roms", "--platform-id", "3"]);
        assert!(parsed.is_err(), "expected clap parse failure");
    }

    #[test]
    fn parse_roms_upload_requires_platform() {
        let parsed = Cli::try_parse_from(["romm-cli", "roms", "upload", "foo.bin"]);
        assert!(
            parsed.is_err(),
            "expected clap parse failure without --platform"
        );
    }

    #[test]
    fn parse_roms_upload_with_platform_and_file() {
        let cli = Cli::parse_from(["romm-cli", "roms", "upload", "--platform", "3ds", "foo.bin"]);
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
