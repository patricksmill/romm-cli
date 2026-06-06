use crate::error::{from_anyhow, RommError};

use crate::client::RommClient;
use crate::commands::{
    api, auth, cache, download, platforms, roms, scan, sync, Commands, OutputFormat,
};
use crate::core::interrupt::InterruptContext;

fn map_anyhow<T>(result: Result<T, anyhow::Error>) -> Result<T, RommError> {
    result.map_err(from_anyhow)
}

/// Execute one non-TUI CLI command.
pub async fn run(
    command: Commands,
    client: &RommClient,
    global_json: bool,
) -> Result<(), RommError> {
    match command {
        Commands::Api(cmd) => {
            let format = OutputFormat::from_flags(global_json, false);
            map_anyhow(api::handle(cmd, client, format).await)
        }
        Commands::Platforms(cmd) => {
            let format = OutputFormat::from_flags(global_json, cmd.json);
            map_anyhow(platforms::handle(cmd, client, format).await)
        }
        Commands::Roms(cmd) => {
            let format = OutputFormat::from_flags(global_json, cmd.json);
            map_anyhow(roms::handle(*cmd, client, format).await)
        }
        Commands::Scan(cmd) => {
            let format = OutputFormat::from_flags(global_json, false);
            let interrupt = InterruptContext::new();
            map_anyhow(scan::handle(cmd, client, format, Some(interrupt)).await)
        }
        Commands::Sync(cmd) => {
            let format = OutputFormat::from_flags(global_json, cmd.json);
            map_anyhow(sync::handle(cmd, client, format).await)
        }
        Commands::Download(cmd) => {
            let interrupt = InterruptContext::new();
            download::handle(cmd, client, Some(interrupt)).await
        }
        Commands::Cache(cmd) => map_anyhow(cache::handle(cmd)),
        Commands::Auth(cmd) => {
            let format = OutputFormat::from_flags(global_json, false);
            map_anyhow(auth::handle(cmd, client, format).await)
        }
        Commands::Init(_) => Err(RommError::Other(
            "internal routing error: init command in CLI frontend".into(),
        )),
        Commands::Tui { .. } => Err(RommError::Other(
            "internal routing error: TUI command in CLI frontend".into(),
        )),
        Commands::Update => {
            let interrupt = InterruptContext::new();
            map_anyhow(crate::commands::update::handle(Some(interrupt)).await)
        }
        Commands::Completions(_) => Err(RommError::Other(
            "internal routing error: completions command in CLI frontend".into(),
        )),
    }
}
