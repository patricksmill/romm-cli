use anyhow::Result;

use crate::client::RommClient;
use crate::config::Config;

/// Execute the interactive TUI frontend.
pub async fn run(client: RommClient, config: Config) -> Result<()> {
    crate::tui::run(client, config).await
}
