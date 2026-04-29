//! Authentication command group (`romm-cli auth ...`).
//!
//! This is intentionally layered on top of the existing config/keyring logic in
//! `src/config.rs` so users can rotate credentials without re-entering the ROM
//! path / base URL.

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use dialoguer::{Input, Password, Select};
use serde_json::json;
use std::fs;
use std::io::Read;

use crate::client::RommClient;
use crate::commands::OutputFormat;
use crate::config::{
    disk_has_unresolved_keyring_sentinel, is_keyring_placeholder, load_config, persist_user_config,
    read_user_config_json_from_disk, user_config_json_path, AuthConfig, Config,
    KEYRING_SECRET_PLACEHOLDER,
};
use crate::endpoints::client_tokens::ExchangeClientToken;

/// Top-level `romm-cli auth` command group.
#[derive(Args, Debug, Clone)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub action: AuthAction,
}

/// Specific action within `romm-cli auth`.
#[derive(Subcommand, Debug, Clone)]
pub enum AuthAction {
    /// Set/rotate authentication credentials (Bearer, Basic, API key, or pairing code).
    Login(AuthLoginCommand),
    /// Remove stored authentication (leaves non-auth config untouched).
    Logout,
    /// Show current authentication mode and where it comes from (env/config/keyring).
    Status,
}

/// Options for `romm-cli auth login`.
///
/// If no auth flags are provided, the command runs interactively.
#[derive(Args, Debug, Clone)]
pub struct AuthLoginCommand {
    /// API token (Bearer). Skips interactive prompts.
    #[arg(long)]
    pub token: Option<String>,

    /// Read API token (Bearer) from UTF-8 file. Use '-' for stdin.
    #[arg(long)]
    pub token_file: Option<String>,

    /// Basic auth username.
    #[arg(long)]
    pub username: Option<String>,

    /// Basic auth password (discouraged: visible in process list).
    #[arg(long)]
    pub password: Option<String>,

    /// Read Basic auth password from a UTF-8 file. Use '-' for stdin.
    #[arg(long)]
    pub password_file: Option<String>,

    /// API key header name (e.g. X-API-Key).
    #[arg(long)]
    pub api_key_header: Option<String>,

    /// API key value.
    #[arg(long)]
    pub api_key: Option<String>,

    /// Web UI pairing code (8 characters).
    #[arg(long)]
    pub pairing_code: Option<String>,
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn read_secret_from_path_or_stdin(path: &str) -> Result<String> {
    let mut content = String::new();
    if path == "-" {
        std::io::stdin()
            .read_to_string(&mut content)
            .context("read secret from stdin")?;
    } else {
        content =
            fs::read_to_string(path).with_context(|| format!("read secret from file {}", path))?;
    }
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("secret read from {} is empty", path));
    }
    Ok(trimmed.to_string())
}

fn disk_config_or_die() -> Result<Config> {
    read_user_config_json_from_disk().ok_or_else(|| {
        anyhow!(
            "Could not read user config.json. Run `romm-cli init` first (or ensure your config exists)."
        )
    })
}

fn preserve_non_auth_fields_for_persist() -> Result<(String, String, bool, std::path::PathBuf)> {
    let disk = disk_config_or_die()?;
    let config_path =
        user_config_json_path().ok_or_else(|| anyhow!("Could not resolve config path"))?;
    Ok((
        disk.base_url,
        disk.download_dir,
        disk.use_https,
        config_path,
    ))
}

async fn persist_auth_from_login(auth: Option<AuthConfig>, client: &RommClient) -> Result<()> {
    let (base_url, download_dir, use_https, config_path) = preserve_non_auth_fields_for_persist()?;

    // Compute the human-readable auth mode before persisting, since `auth` is moved.
    let mode = match &auth {
        None => "none",
        Some(AuthConfig::Basic { .. }) => "basic",
        Some(AuthConfig::Bearer { .. }) => "bearer",
        Some(AuthConfig::ApiKey { .. }) => "api-key",
    };

    persist_user_config(&base_url, &download_dir, use_https, auth)?;

    if config_path.exists() {
        println!("Auth updated: {mode} (wrote {})", config_path.display());
    } else {
        // Should not happen (persist_user_config creates the directory), but keep it safe.
        println!("Auth updated: {mode}");
    }

    // Keep the client reference "used" in this helper for future extensions
    // (e.g. verification) without changing function signature.
    let _ = client.verbose();
    Ok(())
}

async fn login_interactive(cmd: &AuthLoginCommand, client: &RommClient) -> Result<AuthConfig> {
    // If any login flags are present, we do not consider it "interactive".
    let has_flags = cmd.token.is_some()
        || cmd.token_file.is_some()
        || cmd.username.is_some()
        || cmd.password.is_some()
        || cmd.password_file.is_some()
        || cmd.api_key_header.is_some()
        || cmd.api_key.is_some()
        || cmd.pairing_code.is_some();
    if has_flags {
        return Err(anyhow!(
            "internal error: interactive auth called with flags present"
        ));
    }

    let items = vec![
        "Basic (username + password)",
        "API Token (Bearer)",
        "API key in custom header",
        "Pair with Web UI (8-character code)",
    ];
    let idx = Select::new()
        .with_prompt("Authentication")
        .items(&items)
        .default(1)
        .interact()?;

    match idx {
        0 => {
            let username: String = Input::new().with_prompt("Username").interact_text()?;
            let password = Password::new().with_prompt("Password").interact()?;
            Ok(AuthConfig::Basic {
                username: username.trim().to_string(),
                password,
            })
        }
        1 => {
            let token = Password::new().with_prompt("API Token").interact()?;
            Ok(AuthConfig::Bearer { token })
        }
        2 => {
            let header: String = Input::new()
                .with_prompt("Header name (e.g. X-API-Key)")
                .interact_text()?;
            let key = Password::new().with_prompt("API key value").interact()?;
            Ok(AuthConfig::ApiKey {
                header: header.trim().to_string(),
                key,
            })
        }
        3 => {
            let code: String = Input::new()
                .with_prompt("8-character pairing code")
                .interact_text()?;

            // Pairing-code exchange should not depend on the current auth mode
            // (because we are rotating it). Use an unauthenticated client.
            let disk = disk_config_or_die()?;
            let temp_config = Config {
                base_url: disk.base_url,
                download_dir: disk.download_dir,
                use_https: disk.use_https,
                auth: None,
            };
            let unauth_client = RommClient::new(&temp_config, client.verbose())?;

            let endpoint = ExchangeClientToken { code };
            let response = unauth_client
                .call(&endpoint)
                .await
                .context("failed to exchange pairing code")?;

            Ok(AuthConfig::Bearer {
                token: response.raw_token,
            })
        }
        _ => Err(anyhow!("unreachable login auth choice")),
    }
}

fn env_hint_auth_mode() -> Option<&'static str> {
    // Mirrors `load_config` auth precedence order at a high level.
    if env_nonempty("API_USERNAME").is_some() || env_nonempty("API_PASSWORD").is_some() {
        return Some("basic");
    }
    if env_nonempty("API_KEY").is_some() || env_nonempty("API_KEY_HEADER").is_some() {
        return Some("api-key");
    }
    if env_nonempty("API_TOKEN").is_some()
        || env_nonempty("ROMM_TOKEN_FILE").is_some()
        || env_nonempty("API_TOKEN_FILE").is_some()
    {
        return Some("bearer");
    }
    None
}

fn disk_secret_unresolved_placeholder(auth: &Option<AuthConfig>) -> bool {
    match auth {
        None => false,
        Some(AuthConfig::Basic { password, .. }) => is_keyring_placeholder(password),
        Some(AuthConfig::Bearer { token }) => is_keyring_placeholder(token),
        Some(AuthConfig::ApiKey { key, .. }) => is_keyring_placeholder(key),
    }
}

fn auth_mode_string(auth: &Option<AuthConfig>) -> &'static str {
    match auth {
        None => "none",
        Some(AuthConfig::Basic { .. }) => "basic",
        Some(AuthConfig::Bearer { .. }) => "bearer",
        Some(AuthConfig::ApiKey { .. }) => "api-key",
    }
}

/// Execute `romm-cli auth ...`.
pub async fn handle(cmd: AuthCommand, client: &RommClient, format: OutputFormat) -> Result<()> {
    match cmd.action {
        AuthAction::Login(login) => {
            // Non-interactive fast path: infer auth from provided flags.
            let has_flags = login.token.is_some()
                || login.token_file.is_some()
                || login.username.is_some()
                || login.password.is_some()
                || login.password_file.is_some()
                || login.api_key_header.is_some()
                || login.api_key.is_some()
                || login.pairing_code.is_some();

            let auth = if has_flags {
                // Enforce "single auth mode" for predictable behavior.
                let mut modes = Vec::new();
                if login.pairing_code.is_some() {
                    modes.push("pairing-code");
                }
                if login.token.is_some() || login.token_file.is_some() {
                    modes.push("bearer");
                }
                if login.username.is_some()
                    || login.password.is_some()
                    || login.password_file.is_some()
                {
                    modes.push("basic");
                }
                if login.api_key_header.is_some() || login.api_key.is_some() {
                    modes.push("api-key");
                }

                if modes.is_empty() {
                    return Err(anyhow!("no authentication fields found"));
                }
                if modes.len() != 1 {
                    return Err(anyhow!(
                        "Specify exactly one authentication mode, got: {}",
                        modes.join(", ")
                    ));
                }

                if let Some(code) = login.pairing_code {
                    let disk = disk_config_or_die()?;
                    let temp_config = Config {
                        base_url: disk.base_url,
                        download_dir: disk.download_dir,
                        use_https: disk.use_https,
                        auth: None,
                    };
                    let unauth_client = RommClient::new(&temp_config, client.verbose())?;
                    let endpoint = ExchangeClientToken { code };
                    let response = unauth_client
                        .call(&endpoint)
                        .await
                        .context("failed to exchange pairing code")?;
                    AuthConfig::Bearer {
                        token: response.raw_token,
                    }
                } else if login.token.is_some() || login.token_file.is_some() {
                    let token = match (login.token, login.token_file) {
                        (Some(_), Some(_)) => {
                            return Err(anyhow!(
                                "Provide either --token or --token-file, not both"
                            ));
                        }
                        (Some(t), None) => t,
                        (None, Some(f)) => read_secret_from_path_or_stdin(&f)?,
                        (None, None) => unreachable!("checked by flags"),
                    };
                    AuthConfig::Bearer { token }
                } else if login.api_key_header.is_some() || login.api_key.is_some() {
                    let header = login.api_key_header.ok_or_else(|| {
                        anyhow!("--api-key-header is required when using --api-key")
                    })?;
                    let key = login.api_key.ok_or_else(|| {
                        anyhow!("--api-key is required when using --api-key-header")
                    })?;
                    AuthConfig::ApiKey {
                        header: header.trim().to_string(),
                        key,
                    }
                } else {
                    // Basic
                    let username = login
                        .username
                        .ok_or_else(|| anyhow!("--username is required for basic auth"))?;
                    let password = match (login.password, login.password_file) {
                        (Some(p), None) => p,
                        (None, Some(f)) => read_secret_from_path_or_stdin(&f)?,
                        (None, None) => {
                            return Err(anyhow!(
                                "--password or --password-file is required for basic auth"
                            ))
                        }
                        (Some(_), Some(_)) => {
                            return Err(anyhow!(
                                "Provide either --password or --password-file, not both"
                            ))
                        }
                    };
                    AuthConfig::Basic {
                        username: username.trim().to_string(),
                        password,
                    }
                }
            } else {
                login_interactive(&login, client).await?
            };

            persist_auth_from_login(Some(auth), client).await?;
            Ok(())
        }

        AuthAction::Logout => {
            persist_auth_from_login(None, client).await?;
            Ok(())
        }

        AuthAction::Status => {
            let effective = load_config()?;
            let disk = read_user_config_json_from_disk();

            let effective_mode = auth_mode_string(&effective.auth);
            let disk_auth = disk.as_ref().and_then(|c| c.auth.clone());
            let disk_mode = auth_mode_string(&disk_auth);
            let disk_unresolved = disk_secret_unresolved_placeholder(&disk_auth);
            let unresolved_keyring_sentinel = disk_has_unresolved_keyring_sentinel(&effective);

            let env_mode = env_hint_auth_mode();
            let env_hints = json!({
                "API_USERNAME_set": std::env::var("API_USERNAME").ok().map(|v| !v.trim().is_empty()).unwrap_or(false),
                "API_PASSWORD_set": std::env::var("API_PASSWORD").ok().map(|v| !v.trim().is_empty()).unwrap_or(false),
                "API_TOKEN_set": std::env::var("API_TOKEN").ok().map(|v| !v.trim().is_empty()).unwrap_or(false),
                "ROMM_TOKEN_FILE_set": std::env::var("ROMM_TOKEN_FILE").ok().map(|v| !v.trim().is_empty()).unwrap_or(false),
                "API_TOKEN_FILE_set": std::env::var("API_TOKEN_FILE").ok().map(|v| !v.trim().is_empty()).unwrap_or(false),
                "API_KEY_HEADER_set": std::env::var("API_KEY_HEADER").ok().map(|v| !v.trim().is_empty()).unwrap_or(false),
                "API_KEY_set": std::env::var("API_KEY").ok().map(|v| !v.trim().is_empty()).unwrap_or(false),
            });

            match format {
                OutputFormat::Json => {
                    let out = json!({
                        "effective": { "mode": effective_mode },
                        "disk": {
                            "mode": disk_mode,
                            "secret_unresolved_in_keyring": disk_unresolved,
                        },
                        "env_hint_mode": env_mode,
                        "env": env_hints,
                        "keyring_resolution_status": {
                            "unresolved_keyring_sentinel": unresolved_keyring_sentinel,
                        }
                    });
                    println!("{}", serde_json::to_string_pretty(&out)?);
                }
                OutputFormat::Text => {
                    println!("Auth (effective): {effective_mode}");
                    println!("Auth (disk): {disk_mode}");
                    if disk_auth.is_some() {
                        println!(
                            "Disk secret unresolved sentinel: {}",
                            if disk_unresolved { "yes" } else { "no" }
                        );
                    } else {
                        println!("Disk config: not found");
                    }
                    if unresolved_keyring_sentinel {
                        println!(
                            "Keyring lookup failed: config contains `{}` but effective auth is missing.",
                            KEYRING_SECRET_PLACEHOLDER
                        );
                    }
                    if let Some(m) = env_mode {
                        println!("Env auth hint (not showing secrets): {m}");
                    } else {
                        println!("Env auth hint: none");
                    }
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Cli, Commands};
    use clap::Parser;

    struct TestEnv {
        dir: std::path::PathBuf,
        _guard: std::sync::MutexGuard<'static, ()>,
    }

    impl TestEnv {
        fn new() -> Self {
            let guard = crate::config::test_env_lock()
                .lock()
                .unwrap_or_else(|e| e.into_inner());

            let mut unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string();
            unique.push_str("-auth");

            let dir = std::env::temp_dir().join(format!("romm-cli-auth-test-{}", unique));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("create test config dir");

            clear_env();
            std::env::set_var("ROMM_TEST_CONFIG_DIR", &dir);
            Self { dir, _guard: guard }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            clear_env();
            let _ = std::fs::remove_dir_all(&self.dir);
            std::env::remove_var("ROMM_TEST_CONFIG_DIR");
        }
    }

    fn clear_env() {
        for key in [
            "ROMM_TEST_CONFIG_DIR",
            "API_BASE_URL",
            "ROMM_ROMS_DIR",
            "ROMM_DOWNLOAD_DIR",
            "API_USE_HTTPS",
            "API_USERNAME",
            "API_PASSWORD",
            "API_TOKEN",
            "ROMM_TOKEN_FILE",
            "API_TOKEN_FILE",
            "API_KEY",
            "API_KEY_HEADER",
        ] {
            std::env::remove_var(key);
        }
    }

    fn write_disk_config(path: &std::path::Path, disk_auth: Option<AuthConfig>) {
        fs::create_dir_all(path).unwrap();
        let cfg = Config {
            base_url: "https://disk.example".to_string(),
            download_dir: "/disk/dl".to_string(),
            use_https: true,
            auth: disk_auth,
        };
        let content = serde_json::to_string_pretty(&cfg).unwrap();
        fs::write(path.join("config.json"), content).unwrap();
    }

    #[test]
    fn parse_auth_logout() {
        let cli = Cli::parse_from(["romm-cli", "auth", "logout"]);
        let Commands::Auth(cmd) = cli.command else {
            panic!("expected auth command");
        };
        assert!(matches!(cmd.action, AuthAction::Logout));
    }

    #[test]
    fn auth_status_unresolved_sentinel_detected_from_disk() {
        let env = TestEnv::new();
        write_disk_config(
            &env.dir,
            Some(AuthConfig::Bearer {
                token: KEYRING_SECRET_PLACEHOLDER.to_string(),
            }),
        );

        let effective = Config {
            base_url: String::new(),
            download_dir: String::new(),
            use_https: true,
            auth: None,
        };

        assert!(disk_has_unresolved_keyring_sentinel(&effective));
    }

    #[test]
    fn auth_login_preserves_disk_non_auth_fields_even_with_env_overrides() {
        let env = TestEnv::new();
        write_disk_config(&env.dir, None);

        std::env::set_var("API_BASE_URL", "https://env.example");
        std::env::set_var("ROMM_ROMS_DIR", "/env/dl");
        std::env::set_var("API_USE_HTTPS", "false");

        // Use the sentinel value to avoid keyring interaction in tests.
        let disk_auth = Some(AuthConfig::Bearer {
            token: KEYRING_SECRET_PLACEHOLDER.to_string(),
        });

        let tmp_client = RommClient::new(
            &Config {
                base_url: "https://dummy.example".to_string(),
                download_dir: "/tmp".to_string(),
                use_https: true,
                auth: None,
            },
            false,
        )
        .unwrap();

        // This helper is async; execute with a runtime.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(persist_auth_from_login(disk_auth, &tmp_client))
            .unwrap();

        let saved = read_user_config_json_from_disk().unwrap();
        assert_eq!(saved.base_url, "https://disk.example");
        assert_eq!(saved.download_dir, "/disk/dl");
        assert!(saved.use_https);
        match saved.auth {
            Some(AuthConfig::Bearer { token }) => {
                assert!(is_keyring_placeholder(&token));
            }
            _ => panic!("expected bearer auth on disk"),
        }
    }

    #[test]
    fn auth_logout_clears_auth_but_preserves_non_auth_fields() {
        let env = TestEnv::new();
        write_disk_config(
            &env.dir,
            Some(AuthConfig::Bearer {
                token: "some-token".to_string(),
            }),
        );

        let tmp_client = RommClient::new(
            &Config {
                base_url: "https://dummy.example".to_string(),
                download_dir: "/tmp".to_string(),
                use_https: true,
                auth: None,
            },
            false,
        )
        .unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(persist_auth_from_login(None, &tmp_client))
            .unwrap();

        let saved = read_user_config_json_from_disk().unwrap();
        assert_eq!(saved.base_url, "https://disk.example");
        assert_eq!(saved.download_dir, "/disk/dl");
        assert!(saved.use_https);
        assert!(saved.auth.is_none());
    }
}
