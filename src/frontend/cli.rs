use anyhow::{anyhow, Result};

use crate::client::RommClient;
use crate::commands::{api, download, platforms, roms, Commands, OutputFormat};

/// Execute one non-TUI CLI command.
pub async fn run(command: Commands, client: &RommClient, global_json: bool) -> Result<()> {
    match command {
        Commands::Api(cmd) => {
            let format = OutputFormat::from_flags(global_json, false);
            api::handle(cmd, client, format).await
        }
        Commands::Platforms(cmd) => {
            let format = OutputFormat::from_flags(global_json, cmd.json);
            platforms::handle(cmd, client, format).await
        }
        Commands::Roms(cmd) => {
            let format = OutputFormat::from_flags(global_json, cmd.json);
            roms::handle(cmd, client, format).await
        }
        Commands::Download(cmd) => download::handle(cmd, client).await,
        Commands::Init(_) => Err(anyhow!(
            "internal routing error: init command in CLI frontend"
        )),
        Commands::Tui => Err(anyhow!(
            "internal routing error: TUI command in CLI frontend"
        )),
        Commands::Update => Err(anyhow!(
            "internal routing error: update command in CLI frontend"
        )),
    }
}
