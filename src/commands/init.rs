//! Interactive `romm-cli init` — writes user-level `romm-cli/.env`.
//!
//! Secrets (passwords, tokens, API keys) are stored in the OS keyring
//! when available, keeping the `.env` file free of plaintext credentials.

use anyhow::{anyhow, Context, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::fs;
use std::io::Write;

use crate::config::{keyring_store, normalize_romm_origin, user_config_env_path};

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
        .with_initial_text("http://")
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

    // ── Build .env lines (secrets go to keyring, not here) ─────────────
    let mut lines: Vec<String> = vec![
        "# romm-cli user configuration".to_string(),
        "# Secrets are stored in the OS keyring when available.".to_string(),
        "# Applied after project .env: only fills variables not already set.".to_string(),
        String::new(),
        format!("API_BASE_URL={}", escape_env_value(&base_url)),
        format!("ROMM_DOWNLOAD_DIR={}", escape_env_value(&download_dir)),
        String::new(),
    ];

    // Track what we stored in the keyring vs .env file.
    let mut keyring_success = true;

    match choice {
        AuthChoice::None => {
            lines.push("# No auth variables set.".to_string());
        }
        AuthChoice::Basic => {
            let username: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Username")
                .interact_text()?;
            let password = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Password")
                .interact()?;

            // Username is not secret, always in .env
            lines.push("# Basic auth (password stored in OS keyring)".to_string());
            lines.push(format!(
                "API_USERNAME={}",
                escape_env_value(username.trim())
            ));

            // Try keyring for password; fall back to .env
            if let Err(e) = keyring_store("API_PASSWORD", &password) {
                eprintln!(
                    "warning: could not store password in OS keyring: {e}\n\
                     Falling back to plaintext .env storage."
                );
                lines.push(format!("API_PASSWORD={}", escape_env_value(&password)));
                keyring_success = false;
            }
        }
        AuthChoice::Bearer => {
            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Bearer token")
                .interact()?;

            lines.push("# Bearer token (stored in OS keyring)".to_string());

            if let Err(e) = keyring_store("API_TOKEN", &token) {
                eprintln!(
                    "warning: could not store token in OS keyring: {e}\n\
                     Falling back to plaintext .env storage."
                );
                lines.push(format!("API_TOKEN={}", escape_env_value(&token)));
                keyring_success = false;
            }
        }
        AuthChoice::ApiKeyHeader => {
            let header: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Header name (e.g. X-API-Key)")
                .interact_text()?;
            let key = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("API key value")
                .interact()?;

            lines.push("# Custom header API key (key stored in OS keyring)".to_string());
            lines.push(format!(
                "API_KEY_HEADER={}",
                escape_env_value(header.trim())
            ));

            if let Err(e) = keyring_store("API_KEY", &key) {
                eprintln!(
                    "warning: could not store API key in OS keyring: {e}\n\
                     Falling back to plaintext .env storage."
                );
                lines.push(format!("API_KEY={}", escape_env_value(&key)));
                keyring_success = false;
            }
        }
    }

    let content = lines.join("\n") + "\n";
    {
        let mut f = fs::File::create(&path).with_context(|| format!("write {}", path.display()))?;
        f.write_all(content.as_bytes())?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    println!("Wrote {}", path.display());
    if keyring_success {
        println!("Credentials stored securely in the OS keyring.");
    }
    println!("You can run `romm-cli tui` or `romm-tui` to start the TUI.");
    Ok(())
}

fn escape_env_value(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.chars()
            .any(|c| c.is_whitespace() || c == '#' || c == '"' || c == '\'');
    if needs_quote {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}
