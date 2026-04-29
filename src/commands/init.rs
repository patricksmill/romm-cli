//! Interactive `romm-cli init` — writes user-level `romm-cli/config.json`.
//!
//! Secrets (passwords, tokens, API keys) are stored in the OS keyring
//! when available, keeping `config.json` free of plaintext credentials when keyring succeeds.

use anyhow::{anyhow, Context, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::fs;
use std::io::Read;

use crate::client::RommClient;
use crate::config::{
    normalize_romm_origin, persist_user_config, user_config_json_path, AuthConfig, Config,
};

#[derive(Args, Debug, Clone)]
pub struct InitCommand {
    /// Overwrite existing user config `config.json` without asking
    #[arg(long)]
    pub force: bool,

    /// Print the path to the user config `config.json` and exit
    #[arg(long)]
    pub print_path: bool,

    /// RomM origin URL (e.g. <https://romm.example>). If provided with a token, skips interactive prompts.
    #[arg(long)]
    pub url: Option<String>,

    /// Bearer token string (discouraged: visible in process list).
    #[arg(long)]
    pub token: Option<String>,

    /// Read Bearer token from a UTF-8 file. Use '-' for stdin.
    #[arg(long)]
    pub token_file: Option<String>,

    /// ROMs directory.
    #[arg(long)]
    pub download_dir: Option<String>,

    /// Disable HTTPS (use HTTP instead).
    #[arg(long)]
    pub no_https: bool,

    /// Verify URL and token by fetching OpenAPI after saving.
    #[arg(long)]
    pub check: bool,
}

enum AuthChoice {
    None,
    Basic,
    Bearer,
    ApiKeyHeader,
    PairingCode,
}

pub async fn handle(cmd: InitCommand, verbose: bool) -> Result<()> {
    let Some(path) = user_config_json_path() else {
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

    let is_non_interactive = cmd.url.is_some() || cmd.token.is_some() || cmd.token_file.is_some();

    if path.exists() && !cmd.force {
        if is_non_interactive {
            return Err(anyhow!(
                "Config file already exists at {}. Use --force to overwrite.",
                path.display()
            ));
        }
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

    // ── Non-interactive quick setup ────────────────────────────────────
    if let Some(url) = cmd.url {
        let token = match (cmd.token, cmd.token_file) {
            (Some(t), _) => Some(t),
            (None, Some(f)) => {
                let mut content = String::new();
                if f == "-" {
                    std::io::stdin()
                        .read_to_string(&mut content)
                        .context("read token from stdin")?;
                } else {
                    content =
                        fs::read_to_string(&f).with_context(|| format!("read token file {}", f))?;
                }
                Some(content.trim().to_string())
            }
            (None, None) => None,
        };

        if token.is_none() {
            return Err(anyhow!("--url requires either --token or --token-file"));
        }

        let base_url = normalize_romm_origin(&url);
        let default_dl_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .join("romm-cli");
        let download_dir = cmd
            .download_dir
            .unwrap_or_else(|| default_dl_dir.display().to_string());
        let use_https = !cmd.no_https;
        let auth = Some(AuthConfig::Bearer {
            token: token.unwrap(),
        });

        persist_user_config(&base_url, &download_dir, use_https, auth.clone())?;
        println!("Wrote {}", path.display());

        if cmd.check {
            let config = Config {
                base_url,
                download_dir,
                use_https,
                auth,
            };
            let client = RommClient::new(&config, verbose)?;
            println!("Checking connection to {}...", config.base_url);
            client
                .fetch_openapi_json()
                .await
                .context("failed to fetch OpenAPI JSON")?;
            println!("Success: connected and fetched OpenAPI spec.");

            println!("Verifying authentication...");
            client
                .call(&crate::endpoints::platforms::ListPlatforms)
                .await
                .context("failed to authenticate or fetch platforms")?;
            println!("Success: authentication verified.");
        }
        return Ok(());
    }

    // ── Interactive setup ──────────────────────────────────────────────
    if cmd.token.is_some() || cmd.token_file.is_some() {
        return Err(anyhow!("--token and --token-file require --url"));
    }

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

    // ── ROMs directory ─────────────────────────────────────────────────
    let default_dl_dir = dirs::download_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
        .join("romm-cli");

    let download_dir: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("ROMs directory")
        .default(default_dl_dir.display().to_string())
        .interact_text()?;

    let download_dir = download_dir.trim().to_string();

    // ── Authentication ─────────────────────────────────────────────────
    let use_https = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Connect over HTTPS?")
        .default(true)
        .interact()?;

    let items = vec![
        "No authentication",
        "Basic (username + password)",
        "API Token (Bearer)",
        "API key in custom header",
        "Pair with Web UI (8-character code)",
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
        4 => AuthChoice::PairingCode,
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
                .with_prompt("API Token")
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
        AuthChoice::PairingCode => {
            let code: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("8-character pairing code")
                .interact_text()?;

            println!("Exchanging pairing code...");
            let temp_config = Config {
                base_url: base_url.clone(),
                download_dir: download_dir.clone(),
                use_https,
                auth: None,
            };
            let client = RommClient::new(&temp_config, verbose)?;
            let endpoint = crate::endpoints::client_tokens::ExchangeClientToken { code };

            let response = client
                .call(&endpoint)
                .await
                .context("failed to exchange pairing code")?;
            println!("Successfully paired device as '{}'", response.name);

            Some(AuthConfig::Bearer {
                token: response.raw_token,
            })
        }
    };

    persist_user_config(&base_url, &download_dir, use_https, auth)?;

    println!("Wrote {}", path.display());
    println!("Secrets are stored in the OS keyring when available (see file comments if plaintext fallback was used).");
    println!("You can run `romm-cli tui` or `romm-tui` to start the TUI.");
    Ok(())
}
