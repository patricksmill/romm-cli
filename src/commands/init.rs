//! Interactive `romm-cli init` — writes user-level `romm-cli/.env`.

use anyhow::{anyhow, Context, Result};
use clap::Args;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::fs;
use std::io::Write;

use crate::config::user_config_env_path;

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

    let base_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("ROMM API base URL")
        .with_initial_text("http://")
        .interact_text()?;

    let base_url = base_url.trim().to_string();
    if base_url.is_empty() {
        return Err(anyhow!("Base URL cannot be empty"));
    }

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

    let mut lines: Vec<String> = vec![
        "# romm-cli user configuration (do not commit secrets)".to_string(),
        "# Applied after project .env: only fills variables not already set.".to_string(),
        String::new(),
        format!("API_BASE_URL={}", escape_env_value(&base_url)),
        String::new(),
    ];

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
            lines.push("# Basic auth".to_string());
            lines.push(format!(
                "API_USERNAME={}",
                escape_env_value(username.trim())
            ));
            lines.push(format!("API_PASSWORD={}", escape_env_value(&password)));
        }
        AuthChoice::Bearer => {
            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Bearer token")
                .interact()?;
            lines.push("# Bearer token (API_TOKEN)".to_string());
            lines.push(format!("API_TOKEN={}", escape_env_value(&token)));
        }
        AuthChoice::ApiKeyHeader => {
            let header: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Header name (e.g. X-API-Key)")
                .interact_text()?;
            let key = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("API key value")
                .interact()?;
            lines.push("# Custom header API key".to_string());
            lines.push(format!(
                "API_KEY_HEADER={}",
                escape_env_value(header.trim())
            ));
            lines.push(format!("API_KEY={}", escape_env_value(&key)));
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
