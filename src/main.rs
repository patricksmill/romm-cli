//! Binary entrypoint for `romm-cli`.
//!
//! This crate is intentionally very small: it wires together
//! - configuration loading,
//! - the HTTP client (`RommClient`),
//! - the top-level CLI command parser, and
//! - the currently selected frontend (TUI or plain CLI).
//!
//! All interesting behavior lives in the library modules under `src/`.

mod client;
mod commands;
mod config;
mod endpoints;
mod tui;
mod types;

use anyhow::Result;
use clap::Parser;
use commands::Cli;
use config::load_config;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env in development (no-op if file missing)
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    let config = load_config()?;

    commands::run(cli, config).await
}
