use crate::cli_presentation::CliPresentation;
use romm_api::error::{from_anyhow, RommError};

use crate::commands::{api, auth, cache, download, platforms, roms, scan, sync, Commands};
use romm_api::client::RommClient;
use romm_api::core::interrupt::InterruptContext;

fn map_anyhow<T>(result: Result<T, anyhow::Error>) -> Result<T, RommError> {
    result.map_err(from_anyhow)
}

/// Execute one non-TUI CLI command.
pub async fn run(
    command: Commands,
    client: &RommClient,
    global_json: bool,
    verbose: bool,
) -> Result<(), RommError> {
    match command {
        Commands::Api(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, false, verbose);
            map_anyhow(api::handle(cmd, client, presentation).await)
        }
        Commands::Platforms(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, cmd.json, verbose);
            map_anyhow(platforms::handle(cmd, client, presentation).await)
        }
        Commands::Roms(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, cmd.json, verbose);
            map_anyhow(roms::handle(*cmd, client, presentation).await)
        }
        Commands::Scan(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, false, verbose);
            let interrupt = InterruptContext::new();
            map_anyhow(scan::handle(cmd, client, presentation, Some(interrupt)).await)
        }
        Commands::Sync(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, cmd.json, verbose);
            map_anyhow(sync::handle(cmd, client, presentation).await)
        }
        Commands::Download(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, false, verbose);
            let interrupt = InterruptContext::new();
            download::handle(cmd, client, presentation, Some(interrupt)).await
        }
        Commands::Cache(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, false, verbose);
            map_anyhow(cache::handle(cmd, presentation))
        }
        Commands::Auth(cmd) => {
            let presentation = CliPresentation::from_cli(global_json, false, verbose);
            map_anyhow(auth::handle(cmd, client, presentation).await)
        }
        Commands::Init(_) => Err(RommError::Other(
            "internal routing error: init command in CLI frontend".into(),
        )),
        Commands::Update => {
            let presentation = CliPresentation::from_cli(global_json, false, verbose);
            let interrupt = InterruptContext::new();
            map_anyhow(crate::commands::update::handle(presentation, Some(interrupt)).await)
        }
        Commands::Completions(_) => Err(RommError::Other(
            "internal routing error: completions command in CLI frontend".into(),
        )),
    }
}
