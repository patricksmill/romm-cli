use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::cli_presentation::CliPresentation;
use crate::commands::OutputFormat;
use romm_api::core::cache::RomCache;

#[derive(Args, Debug)]
pub struct CacheCommand {
    #[command(subcommand)]
    pub action: CacheAction,
}

#[derive(Subcommand, Debug)]
pub enum CacheAction {
    /// Print the effective ROM cache file path.
    Path,
    /// Show ROM cache metadata and parse status.
    Info,
    /// Delete the ROM cache file if it exists.
    Clear,
}

#[derive(Serialize)]
struct CacheInfoJson {
    path: String,
    exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_error: Option<String>,
}

pub fn handle(cmd: CacheCommand, presentation: CliPresentation) -> Result<()> {
    let format = presentation.format;
    match cmd.action {
        CacheAction::Path => {
            println!("{}", RomCache::effective_path().display());
        }
        CacheAction::Info => {
            let info = RomCache::read_info();
            match format {
                OutputFormat::Json => {
                    let out = CacheInfoJson {
                        path: info.path.display().to_string(),
                        exists: info.exists,
                        size_bytes: info.size_bytes,
                        version: info.version,
                        entries: info.entry_count,
                        parse_error: info.parse_error,
                    };
                    println!("{}", serde_json::to_string_pretty(&out)?);
                }
                OutputFormat::Text => {
                    println!("path: {}", info.path.display());
                    println!("exists: {}", info.exists);
                    if let Some(size) = info.size_bytes {
                        println!("size_bytes: {size}");
                    }
                    if let Some(version) = info.version {
                        println!("version: {version}");
                    }
                    if let Some(count) = info.entry_count {
                        println!("entries: {count}");
                    }
                    if let Some(err) = info.parse_error {
                        println!("parse_error: {err}");
                    }
                }
            }
        }
        CacheAction::Clear => {
            if presentation.is_json() {
                let cleared = RomCache::clear_file()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "cleared": cleared }))?
                );
            } else if RomCache::clear_file()? {
                println!("ROM cache cleared.");
            } else {
                println!("ROM cache file does not exist.");
            }
        }
    }
    Ok(())
}
