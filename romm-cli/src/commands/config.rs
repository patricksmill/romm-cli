//! `config` — inspect and persist local configuration (approach D).

use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use dialoguer::Confirm;
use serde_json::json;

use crate::cli_presentation::CliPresentation;
use crate::commands::OutputFormat;
use romm_api::config::{
    env_var_for_key, env_var_for_platform_key, load_config_with_sources, persist_user_config,
    read_user_config_json_from_disk, redact_config, reset_user_config, set_config_key,
    user_config_json_path, ConfigSources,
};

#[derive(Args, Debug)]
#[command(
    about = "Inspect and update local configuration (file + env precedence)",
    after_help = "Examples:\n  \
      romm-cli config path\n  \
      romm-cli config show --sources --json\n  \
      romm-cli config set theme dracula\n  \
      romm-cli config env-map save_sync.device_id\n  \
      romm-cli config reset --yes"
)]
pub struct ConfigCommand {
    /// Output as JSON (overrides global --json when set).
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show effective config, on-disk file, or per-field sources.
    Show {
        /// Print on-disk config.json only (no env merge).
        #[arg(long)]
        file: bool,
        /// Include per-field source attribution (implies effective merge).
        #[arg(long)]
        sources: bool,
        /// Show auth secrets (TTY confirmation required).
        #[arg(long)]
        reveal_secrets: bool,
    },
    /// Set one config field in config.json (dotted key path).
    Set { key: String, value: String },
    /// Print environment variable name(s) for a config key.
    EnvMap { key: Option<String> },
    /// Print path to config.json.
    Path,
    /// Delete config.json and clear keyring secrets.
    Reset {
        #[arg(long)]
        yes: bool,
    },
}

pub fn handle(cmd: ConfigCommand, presentation: CliPresentation) -> Result<()> {
    let format = presentation.format;
    match cmd.action {
        ConfigAction::Path => match user_config_json_path() {
            Some(p) => println!("{}", p.display()),
            None => return Err(anyhow!("config directory not found")),
        },
        ConfigAction::EnvMap { key } => match key {
            Some(k) => {
                if let Some(v) = env_var_for_key(&k) {
                    println!("{v}");
                } else if let Some(v) = env_var_for_platform_key(&k) {
                    println!("{v}");
                } else {
                    return Err(anyhow!("no environment variable mapped for key {k:?}"));
                }
            }
            None => {
                let keys = [
                    "base_url",
                    "download_dir",
                    "use_https",
                    "theme",
                    "extras_defaults.include_related_roms",
                    "extras_defaults.include_cover",
                    "extras_defaults.include_manual",
                    "save_sync.save_dir",
                    "save_sync.device_id",
                ];
                for k in keys {
                    if let Some(v) = env_var_for_key(k) {
                        println!("{k}\t{v}");
                    }
                }
                println!("save_sync.platform_dirs.<id>\tROMM_SAVE_SYNC_PLATFORM_DIR_<id>");
                println!("roms_layout.platform_dirs.<id>\tROMM_ROMS_PLATFORM_DIR_<id>");
            }
        },
        ConfigAction::Reset { yes } => {
            if !yes {
                let ok = Confirm::new()
                    .with_prompt("Delete config.json and clear keyring secrets?")
                    .default(false)
                    .interact()?;
                if !ok {
                    return Ok(());
                }
            }
            reset_user_config().map_err(|e| anyhow!("{e}"))?;
            match format {
                OutputFormat::Json => println!(r#"{{"reset": true}}"#),
                OutputFormat::Text => println!("Configuration reset."),
            }
        }
        ConfigAction::Set { key, value } => {
            let mut config = read_user_config_json_from_disk().ok_or_else(|| {
                anyhow!("no config.json found — run `romm-cli init` or set base_url first")
            })?;
            set_config_key(&mut config, &key, &value).map_err(|e| anyhow!("{e}"))?;
            persist_user_config(&config).map_err(|e| anyhow!("{e}"))?;
            match format {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "key": key,
                            "value": value,
                            "path": user_config_json_path().map(|p| p.display().to_string()),
                        }))?
                    );
                }
                OutputFormat::Text => {
                    println!("Updated {key} in config.json");
                }
            }
        }
        ConfigAction::Show {
            file,
            sources,
            reveal_secrets,
        } => {
            if file {
                let disk = read_user_config_json_from_disk()
                    .ok_or_else(|| anyhow!("config.json not found"))?;
                let out = if reveal_secrets {
                    confirm_reveal_secrets()?;
                    disk
                } else {
                    redact_config(&disk)
                };
                print_config(&out, None, format)?;
            } else if sources {
                let (config, src) = load_config_with_sources().map_err(|e| anyhow!("{e}"))?;
                let out = if reveal_secrets {
                    confirm_reveal_secrets()?;
                    config
                } else {
                    redact_config(&config)
                };
                print_config(&out, Some(&src), format)?;
            } else {
                let (config, _) = load_config_with_sources().map_err(|e| anyhow!("{e}"))?;
                let out = if reveal_secrets {
                    confirm_reveal_secrets()?;
                    config
                } else {
                    redact_config(&config)
                };
                print_config(&out, None, format)?;
            }
        }
    }
    Ok(())
}

fn confirm_reveal_secrets() -> Result<()> {
    use std::io::IsTerminal;
    if !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal()) {
        return Err(anyhow!(
            "--reveal-secrets requires an interactive terminal; omit the flag in scripts"
        ));
    }
    let ok = Confirm::new()
        .with_prompt("Reveal auth secrets in output?")
        .default(false)
        .interact()?;
    if !ok {
        return Err(anyhow!("cancelled"));
    }
    Ok(())
}

fn print_config(
    config: &romm_api::config::Config,
    sources: Option<&ConfigSources>,
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Json => {
            if let Some(sources) = sources {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "config": config,
                        "sources": sources,
                    }))?
                );
            } else {
                println!("{}", serde_json::to_string_pretty(config)?);
            }
        }
        OutputFormat::Text => {
            if let Some(sources) = sources {
                println!("{}", serde_json::to_string_pretty(config)?);
                println!();
                println!("sources: {}", serde_json::to_string_pretty(sources)?);
            } else {
                println!("{}", serde_json::to_string_pretty(config)?);
            }
        }
    }
    Ok(())
}
