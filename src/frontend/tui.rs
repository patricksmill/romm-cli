use anyhow::Result;

use crate::client::RommClient;
use crate::config::Config;

/// Execute the interactive TUI frontend (config must already be loaded).
pub async fn run(client: RommClient, config: Config) -> Result<()> {
    crate::tui::run(client, config).await
}

/// Load layered env, optional first-time setup, then run the TUI.
pub async fn run_interactive(verbose: bool) -> Result<()> {
    crate::tui::run_interactive(verbose).await
}
