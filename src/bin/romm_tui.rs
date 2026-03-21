//! Launches the ROMM TUI only (no `tui` subcommand). Same config as `romm-cli`.

use anyhow::Result;
use romm_cli::client::RommClient;
use romm_cli::config::{load_config, load_layered_env};
use romm_cli::frontend::tui;

#[tokio::main]
async fn main() -> Result<()> {
    load_layered_env();

    let verbose = std::env::var("ROMM_VERBOSE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let config = load_config()?;
    let client = RommClient::new(&config, verbose)?;
    tui::run(client, config).await
}
