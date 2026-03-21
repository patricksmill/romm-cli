//! Binary entrypoint for `romm-cli`.

use anyhow::Result;
use clap::Parser;
use romm_cli::commands::init;
use romm_cli::commands::{run, Cli, Commands};
use romm_cli::config::{load_config, load_layered_env};

#[tokio::main]
async fn main() -> Result<()> {
    load_layered_env();

    let Cli {
        verbose,
        json,
        command,
    } = Cli::parse();

    match command {
        Commands::Init(cmd) => init::handle(cmd),
        command => {
            let config = load_config()?;
            run(
                Cli {
                    verbose,
                    json,
                    command,
                },
                config,
            )
            .await
        }
    }
}
