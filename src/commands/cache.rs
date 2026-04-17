use anyhow::Result;
use clap::{Args, Subcommand};

use crate::core::cache::RomCache;

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

pub fn handle(cmd: CacheCommand) -> Result<()> {
    match cmd.action {
        CacheAction::Path => {
            println!("{}", RomCache::effective_path().display());
        }
        CacheAction::Info => {
            let info = RomCache::read_info();
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
        CacheAction::Clear => {
            if RomCache::clear_file()? {
                println!("ROM cache cleared.");
            } else {
                println!("ROM cache file does not exist.");
            }
        }
    }
    Ok(())
}
