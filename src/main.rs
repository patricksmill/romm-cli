//! Binary entrypoint for `romm-cli`.

use anyhow::Result;
use clap::Parser;
use romm_cli::commands::init;
use romm_cli::commands::{run, Cli, Commands};
use romm_cli::config::{load_config, load_layered_env};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run_app() -> Result<()> {
    load_layered_env();

    let Cli {
        verbose,
        json,
        command,
    } = Cli::parse();

    let filter = if verbose {
        EnvFilter::new("romm_cli=debug")
    } else {
        EnvFilter::new("romm_cli=info")
    };

    #[cfg(feature = "tui")]
    let is_tui = matches!(command, Commands::Tui);
    #[cfg(not(feature = "tui"))]
    let is_tui = false;

    if !is_tui {
        fmt()
            .with_env_filter(filter.clone())
            .with_writer(std::io::stderr)
            .init();
    }

    match command {
        Commands::Init(cmd) => init::handle(cmd),
        Commands::Update => romm_cli::commands::update::handle(),
        #[cfg(feature = "tui")]
        Commands::Tui => {
            if verbose {
                fmt()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .init();
            }
            romm_cli::frontend::tui::run_interactive(verbose).await
        }
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
