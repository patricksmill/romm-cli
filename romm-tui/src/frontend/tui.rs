use anyhow::Result;

use romm_api::client::RommClient;
use romm_api::config::Config;

/// Execute the interactive TUI frontend (config must already be loaded).
pub async fn run(client: RommClient, config: Config, mock_update: bool) -> Result<()> {
    crate::tui::run(client, config, mock_update).await
}

/// Load layered env, optional first-time setup, then run the TUI.
pub async fn run_interactive(verbose: bool, mock_update: bool) -> Result<()> {
    crate::tui::run_interactive(verbose, mock_update).await
}
