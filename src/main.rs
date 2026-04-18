//! Binary entrypoint for `romm-cli`.

use anyhow::Result;
use clap::Parser;
use romm_cli::commands::init;
use romm_cli::commands::{run, Cli, Commands};
use romm_cli::config::load_config;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run_app() -> Result<()> {
    let Cli {
        verbose,
        json,
        command,
    } = Cli::parse();

    let filter = EnvFilter::from_default_env();

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
        Commands::Init(cmd) => init::handle(cmd, verbose).await,
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
            if !command_requires_config(&command) {
                let dummy_config = romm_cli::config::Config {
                    base_url: String::new(),
                    download_dir: String::new(),
                    use_https: true,
                    auth: None,
                };
                return run(
                    Cli {
                        verbose,
                        json,
                        command,
                    },
                    dummy_config,
                )
                .await;
            }

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

fn command_requires_config(command: &Commands) -> bool {
    match command {
        Commands::Api(_) | Commands::Platforms(_) | Commands::Roms(_) | Commands::Download(_) => {
            true
        }
        Commands::Cache(_) | Commands::Update => false,
        Commands::Init(_) | Commands::Tui => false,
    }
}
