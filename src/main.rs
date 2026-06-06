//! Binary entrypoint for `romm-cli`.

use clap::Parser;
use romm_cli::commands::completions;
use romm_cli::commands::init;
use romm_cli::commands::{run, Cli, Commands};
use romm_cli::config::{load_config, should_check_updates};
use romm_cli::error::{exit_code, from_anyhow, user_message, RommError};
use std::io::{self, IsTerminal, Write};
use std::time::Duration;
use tracing_subscriber::{fmt, EnvFilter};

fn map_anyhow<T>(result: Result<T, anyhow::Error>) -> Result<T, RommError> {
    result.map_err(from_anyhow)
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Error: {}", user_message(&e));
        std::process::exit(exit_code(&e));
    }
}

async fn run_app() -> Result<(), RommError> {
    let Cli {
        verbose,
        json,
        command,
    } = Cli::parse();

    let filter = EnvFilter::from_default_env();

    #[cfg(feature = "tui")]
    let is_tui = matches!(command, Commands::Tui { .. });
    #[cfg(not(feature = "tui"))]
    let is_tui = false;

    if !is_tui {
        fmt()
            .with_env_filter(filter.clone())
            .with_writer(std::io::stderr)
            .init();
    }

    maybe_prompt_for_startup_update(&command).await?;

    match command {
        Commands::Completions(cmd) => completions::handle(cmd),
        Commands::Init(cmd) => map_anyhow(init::handle(cmd, verbose).await),
        #[cfg(feature = "tui")]
        Commands::Tui { mock_update } => {
            if verbose {
                fmt()
                    .with_env_filter(filter)
                    .with_writer(std::io::stderr)
                    .init();
            }
            map_anyhow(romm_cli::frontend::tui::run_interactive(verbose, mock_update).await)
        }
        command => {
            if !command_requires_config(&command) {
                let dummy_config = romm_cli::config::Config {
                    base_url: String::new(),
                    download_dir: String::new(),
                    use_https: true,
                    auth: None,
                    extras_defaults: romm_cli::config::ExtrasDefaults::default(),
                    save_sync: Default::default(),
                    roms_layout: Default::default(),
                    theme: romm_cli::config::default_theme_id(),
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

fn is_interactive_terminal() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn should_skip_startup_update_check(command: &Commands) -> bool {
    // Skip for `update` (redundant) and `tui` (has its own graphical update prompt).
    matches!(
        command,
        Commands::Update | Commands::Tui { .. } | Commands::Completions(_)
    )
}

fn read_update_choice() -> Result<String, RommError> {
    print!(
        "New romm-cli version is available.\n\
         Choose: [u]pdate now, [c]hangelog, [s]kip (default: s): "
    );
    io::stdout()
        .flush()
        .map_err(|e| RommError::Other(e.to_string()))?;
    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .map_err(|e| RommError::Other(e.to_string()))?;
    Ok(choice.trim().to_lowercase())
}

async fn maybe_prompt_for_startup_update(command: &Commands) -> Result<(), RommError> {
    if should_skip_startup_update_check(command)
        || !should_check_updates()
        || !is_interactive_terminal()
    {
        return Ok(());
    }

    let check =
        match tokio::time::timeout(Duration::from_secs(2), romm_cli::update::check_for_update())
            .await
        {
            Ok(Ok(status)) => status,
            Ok(Err(_)) | Err(_) => return Ok(()),
        };

    if !check.should_update {
        return Ok(());
    }

    loop {
        println!(
            "\nUpdate available: current {} -> latest {}",
            check.current_version, check.latest_version
        );
        let choice = read_update_choice()?;
        match choice.as_str() {
            "u" | "update" => {
                let options = romm_cli::update::ApplyUpdateOptions {
                    show_progress: true,
                    show_output: true,
                    no_confirm: true,
                    target_version_tag: Some(check.release_tag.clone()),
                };
                match map_anyhow(romm_cli::update::apply_update(None, options).await)? {
                    romm_cli::update::ApplyUpdateOutcome::Updated(version) => {
                        println!("Updated successfully to `{version}`.");
                        println!("Restart romm-cli to use the new version.");
                    }
                    romm_cli::update::ApplyUpdateOutcome::UpToDate(version) => {
                        println!("Already up to date (`{version}`).");
                    }
                }
                return Ok(());
            }
            "c" | "changelog" => {
                if let Err(err) = romm_cli::update::open_changelog_in_browser() {
                    eprintln!("Could not open changelog: {err:#}");
                } else {
                    println!("Opened changelog: {}", check.changelog_url);
                }
            }
            "" | "s" | "skip" => return Ok(()),
            _ => {
                eprintln!("Unrecognized choice. Enter u, c, or s.");
            }
        }
    }
}

fn command_requires_config(command: &Commands) -> bool {
    match command {
        Commands::Api(_)
        | Commands::Platforms(_)
        | Commands::Roms(_)
        | Commands::Download(_)
        | Commands::Scan(_)
        | Commands::Sync(_) => true,
        Commands::Cache(_) | Commands::Auth(_) | Commands::Update | Commands::Completions(_) => {
            false
        }
        Commands::Init(_) | Commands::Tui { .. } => false,
    }
}
