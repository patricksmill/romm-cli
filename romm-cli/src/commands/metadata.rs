//! `roms metadata` — search, match, edit, unmatch ROM game metadata.

use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use clap::{Args, Subcommand};
use dialoguer::{Confirm, Select};

use crate::cli_presentation::CliPresentation;
use crate::commands::OutputFormat;
use romm_api::client::RommClient;
use romm_api::core::metadata::{
    invalidate_platform_rom_cache, search_covers, search_metadata_matches, search_row_apply_fields,
};
use romm_api::endpoints::roms::{PutRom, RomUpdateFields};
use romm_api::types::metadata::{RomMatchFields, SearchRom};

#[derive(Args, Debug)]
#[command(
    about = "Search, match, and edit game metadata (not per-user props)",
    after_help = "Examples:\n  \
      romm-cli roms metadata search 42 --query zelda\n  \
      romm-cli roms metadata match 42 --igdb-id 1234\n  \
      romm-cli roms metadata match 42 --query zelda --pick\n  \
      romm-cli roms metadata edit 42 --summary \"My note\"\n  \
      romm-cli roms metadata unmatch 42 --yes"
)]
pub struct MetadataCommand {
    #[command(subcommand)]
    pub action: MetadataAction,
}

#[derive(Subcommand, Debug)]
pub enum MetadataAction {
    /// Search metadata providers for match candidates
    Search {
        rom_id: u64,
        #[arg(long, visible_aliases = ["query", "q"])]
        query: Option<String>,
        #[arg(long, default_value = "name")]
        search_by: String,
    },
    /// Apply a metadata provider match (`PUT /api/roms/{id}`)
    Match {
        rom_id: u64,
        #[arg(long, visible_aliases = ["query", "q"])]
        query: Option<String>,
        #[arg(long, default_value = "name")]
        search_by: String,
        /// Interactive picker (default when no ID flags and stdout is a TTY)
        #[arg(long)]
        pick: bool,
        #[arg(long)]
        igdb_id: Option<i64>,
        #[arg(long)]
        moby_id: Option<i64>,
        #[arg(long)]
        ss_id: Option<i64>,
        #[arg(long)]
        launchbox_id: Option<i64>,
        #[arg(long)]
        flashpoint_id: Option<String>,
        #[arg(long)]
        sgdb_id: Option<i64>,
        #[arg(long)]
        ra_id: Option<i64>,
        #[arg(long)]
        hasheous_id: Option<i64>,
        #[arg(long)]
        tgdb_id: Option<i64>,
        #[arg(long)]
        hltb_id: Option<i64>,
        /// After match, optionally set cover from SteamGridDB search using this term
        #[arg(long)]
        cover_query: Option<String>,
    },
    /// Edit name, summary, or cover URL / artwork file
    Edit {
        rom_id: u64,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long)]
        url_cover: Option<String>,
        #[arg(long)]
        artwork: Option<PathBuf>,
    },
    /// Clear provider metadata links (`?unmatch_metadata=true`)
    Unmatch {
        rom_id: u64,
        #[arg(long)]
        yes: bool,
    },
    /// Remove stored cover image (`?remove_cover=true`)
    RemoveCover {
        rom_id: u64,
        #[arg(long)]
        yes: bool,
    },
}

pub async fn handle(
    cmd: MetadataCommand,
    client: &RommClient,
    presentation: CliPresentation,
) -> Result<()> {
    let format = presentation.format;
    match cmd.action {
        MetadataAction::Search {
            rom_id,
            query,
            search_by,
        } => {
            let rows = search_metadata_matches(client, rom_id, query, Some(search_by)).await?;
            print_search_results(&rows, format)?;
        }
        MetadataAction::Match {
            rom_id,
            query,
            search_by,
            pick,
            igdb_id,
            moby_id,
            ss_id,
            launchbox_id,
            flashpoint_id,
            sgdb_id,
            ra_id,
            hasheous_id,
            tgdb_id,
            hltb_id,
            cover_query,
        } => {
            let match_fields = resolve_match_fields(
                client,
                rom_id,
                query,
                search_by,
                pick,
                RomMatchFields {
                    igdb_id,
                    moby_id,
                    ss_id,
                    launchbox_id,
                    flashpoint_id,
                    sgdb_id,
                    ra_id,
                    hasheous_id,
                    tgdb_id,
                    hltb_id,
                    ..Default::default()
                },
            )
            .await?;
            let mut fields = match_fields;
            if let Some(ref cq) = cover_query {
                if let Some(url) = pick_cover_url(client, cq, presentation).await? {
                    fields.url_cover = Some(url);
                }
            }
            let updated = apply_update(client, rom_id, fields, false, false, None).await?;
            print_update(&updated, format)?;
        }
        MetadataAction::Edit {
            rom_id,
            name,
            summary,
            url_cover,
            artwork,
        } => {
            if name.is_none() && summary.is_none() && url_cover.is_none() && artwork.is_none() {
                bail!("Provide at least one of --name, --summary, --url-cover, or --artwork.");
            }
            let updated = apply_update(
                client,
                rom_id,
                RomUpdateFields {
                    name,
                    summary,
                    url_cover,
                    ..Default::default()
                },
                false,
                false,
                artwork,
            )
            .await?;
            print_update(&updated, format)?;
        }
        MetadataAction::Unmatch { rom_id, yes } => {
            if !yes && presentation.is_text() {
                let ok = Confirm::new()
                    .with_prompt(format!("Unmatch metadata for ROM {rom_id}?"))
                    .interact()?;
                if !ok {
                    return Ok(());
                }
            }
            let updated = apply_update(
                client,
                rom_id,
                RomUpdateFields::default(),
                false,
                true,
                None,
            )
            .await?;
            print_update(&updated, format)?;
        }
        MetadataAction::RemoveCover { rom_id, yes } => {
            if !yes && presentation.is_text() {
                let ok = Confirm::new()
                    .with_prompt(format!("Remove cover for ROM {rom_id}?"))
                    .interact()?;
                if !ok {
                    return Ok(());
                }
            }
            let updated = client.update_rom(&PutRom::remove_cover(rom_id)).await?;
            invalidate_platform_rom_cache(updated.platform_id);
            print_update(&updated, format)?;
        }
    }
    Ok(())
}

async fn resolve_match_fields(
    client: &RommClient,
    rom_id: u64,
    query: Option<String>,
    search_by: String,
    pick: bool,
    explicit: RomMatchFields,
) -> Result<RomUpdateFields> {
    if !explicit.is_empty() {
        return Ok(RomUpdateFields {
            match_fields: explicit,
            ..Default::default()
        });
    }
    let use_picker = pick || (std::io::stdout().is_terminal() && query.is_some());
    if !use_picker {
        bail!(
            "Provide a provider id flag (--igdb-id, --ss-id, …) or use --query with --pick for interactive selection."
        );
    }
    let q = query.ok_or_else(|| anyhow!("--query is required for interactive match"))?;
    let rows = search_metadata_matches(client, rom_id, Some(q), Some(search_by)).await?;
    if rows.is_empty() {
        bail!("No metadata matches found.");
    }
    let idx = pick_search_index(&rows)?;
    Ok(search_row_apply_fields(&rows[idx]))
}

fn pick_search_index(rows: &[SearchRom]) -> Result<usize> {
    if rows.len() == 1 {
        return Ok(0);
    }
    let labels: Vec<String> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "{}. {} [igdb:{:?} ss:{:?} moby:{:?}]",
                i + 1,
                r.name,
                r.igdb_id,
                r.ss_id,
                r.moby_id
            )
        })
        .collect();
    let sel = Select::new()
        .with_prompt("Select metadata match")
        .items(&labels)
        .default(0)
        .interact()?;
    Ok(sel)
}

async fn pick_cover_url(
    client: &RommClient,
    query: &str,
    presentation: CliPresentation,
) -> Result<Option<String>> {
    let covers = search_covers(client, query).await?;
    if covers.is_empty() {
        if presentation.is_text() {
            eprintln!("No SteamGridDB covers found for {query:?}; skipping cover.");
        }
        return Ok(None);
    }
    let mut options: Vec<(String, String)> = Vec::new();
    for cover in &covers {
        for res in &cover.resources {
            if let Some(ref url) = res.url {
                let label = format!(
                    "{} {}x{}",
                    cover.name,
                    res.width.unwrap_or(0),
                    res.height.unwrap_or(0)
                );
                options.push((label, url.clone()));
            }
        }
    }
    if options.is_empty() {
        return Ok(None);
    }
    if options.len() == 1 {
        return Ok(Some(options[0].1.clone()));
    }
    let labels: Vec<String> = options.iter().map(|(l, _)| l.clone()).collect();
    let sel = Select::new()
        .with_prompt("Select cover")
        .items(&labels)
        .default(0)
        .interact()?;
    Ok(Some(options[sel].1.clone()))
}

async fn apply_update(
    client: &RommClient,
    rom_id: u64,
    fields: RomUpdateFields,
    remove_cover: bool,
    unmatch_metadata: bool,
    artwork: Option<PathBuf>,
) -> Result<romm_api::types::metadata::RomUpdateResponse> {
    let ep = PutRom {
        rom_id,
        fields,
        remove_cover,
        unmatch_metadata,
        artwork,
    };
    let updated = client.update_rom(&ep).await?;
    invalidate_platform_rom_cache(updated.platform_id);
    Ok(updated)
}

fn print_search_results(rows: &[SearchRom], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(rows)?),
        OutputFormat::Text => {
            if rows.is_empty() {
                println!("No matches.");
            } else {
                for (i, r) in rows.iter().enumerate() {
                    println!(
                        "{}. {} (igdb:{:?} ss:{:?} moby:{:?})",
                        i + 1,
                        r.name,
                        r.igdb_id,
                        r.ss_id,
                        r.moby_id
                    );
                }
            }
        }
    }
    Ok(())
}

fn print_update(
    updated: &romm_api::types::metadata::RomUpdateResponse,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(updated)?),
        OutputFormat::Text => {
            println!(
                "Updated ROM {} (platform {}): {}",
                updated.id,
                updated.platform_id,
                updated.name.as_deref().unwrap_or("—")
            );
        }
    }
    Ok(())
}
