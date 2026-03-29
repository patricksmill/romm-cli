//! Interactive `romm-cli init` — writes user-level `romm-cli/.env`.
//!
//! Secrets (passwords, tokens, API keys) are stored in the OS keyring
//! when available, keeping the `.env` file free of plaintext credentials.

use anyhow::{anyhow, Context, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::fs;

use crate::config::{normalize_romm_origin, persist_user_config, user_config_env_path, AuthConfig};

#[derive(Args, Debug, Clone)]
pub struct InitCommand {
    /// Overwrite existing user config `.env` without asking
    #[arg(long)]
    pub force: bool,

    /// Print the path to the user config `.env` and exit
    #[arg(long)]
    pub print_path: bool,
}

enum AuthChoice {
    None,
    Basic,
    Bearer,
    ApiKeyHeader,
}

pub fn handle(cmd: InitCommand) -> Result<()> {
    let Some(path) = user_config_env_path() else {
        return Err(anyhow!(
            "Could not determine config directory (no HOME / APPDATA?)."
        ));
    };

    if cmd.print_path {
        println!("{}", path.display());
        return Ok(());
    }

    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("invalid config path"))?;

    if path.exists() && !cmd.force {
        let cont = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Overwrite existing config at {}?", path.display()))
            .default(false)
            .interact()?;
        if !cont {
            println!("Aborted.");
            return Ok(());
        }
    }

    fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;

    let base_input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("RomM web URL (same as in your browser; do not add /api)")
        .with_initial_text("https://")
        .interact_text()?;

    let base_input = base_input.trim();
    if base_input.is_empty() {
        return Err(anyhow!("Base URL cannot be empty"));
    }

    let had_api_path = base_input.trim_end_matches('/').ends_with("/api");
    let base_url = normalize_romm_origin(base_input);
    if had_api_path {
        println!(
            "Using `{base_url}` — `/api` was removed. Requests use `/api/...` under that origin automatically."
        );
    }

    // ── Download directory ──────────────────────────────────────────────
    let default_dl_dir = dirs::download_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
        .join("romm-cli");

    let download_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Download directory for ROMs")
        .default(default_dl_dir.display().to_string())
        .interact_text()?;

    let download_dir = download_dir.trim().to_string();

    // ── Authentication ─────────────────────────────────────────────────
    let items = vec![
        "No authentication",
        "Basic (username + password)",
        "Bearer token",
        "API key in custom header",
    ];
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Authentication")
        .items(&items)
        .default(0)
        .interact()?;

    let choice = match idx {
        0 => AuthChoice::None,
        1 => AuthChoice::Basic,
        2 => AuthChoice::Bearer,
        3 => AuthChoice::ApiKeyHeader,
        _ => AuthChoice::None,
    };

    let auth: Option<AuthConfig> = match choice {
        AuthChoice::None => None,
        AuthChoice::Basic => {
            let username: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Username")
                .interact_text()?;
            let password = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Password")
                .interact()?;
            Some(AuthConfig::Basic {
                username: username.trim().to_string(),
                password,
            })
        }
        AuthChoice::Bearer => {
            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Bearer token")
                .interact()?;
            Some(AuthConfig::Bearer { token })
        }
        AuthChoice::ApiKeyHeader => {
            let header: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Header name (e.g. X-API-Key)")
                .interact_text()?;
            let key = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("API key value")
                .interact()?;
            Some(AuthConfig::ApiKey {
                header: header.trim().to_string(),
                key,
            })
        }
    };

    persist_user_config(&base_url, &download_dir, auth)?;

    println!("Wrote {}", path.display());
    println!("Secrets are stored in the OS keyring when available (see file comments if plaintext fallback was used).");
    println!("You can run `romm-cli tui` or `romm-tui` to start the TUI.");
    Ok(())
}
