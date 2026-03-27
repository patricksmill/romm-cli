//! Configuration and authentication for the ROMM client.
//!
//! This module is deliberately independent of any particular frontend:
//! both the TUI and the command-line subcommands share the same `Config`
//! and `AuthConfig` types.
//!
//! ## Environment file precedence
//!
//! Call [`load_layered_env`] before reading config:
//!
//! 1. Variables already set in the process environment (highest priority).
//! 2. Project `.env` in the current working directory (via `dotenvy`).
//! 3. User config: `{config_dir}/romm-cli/.env` — fills keys not already set (so a repo `.env` wins over user defaults).
//! 4. OS keyring — secrets stored by `romm-cli init` (lowest priority fallback).

use std::path::PathBuf;

use anyhow::{anyhow, Result};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum AuthConfig {
    Basic { username: String, password: String },
    Bearer { token: String },
    ApiKey { header: String, key: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub auth: Option<AuthConfig>,
}

fn is_placeholder(value: &str) -> bool {
    value.contains("your-") || value.contains("placeholder") || value.trim().is_empty()
}

// ---------------------------------------------------------------------------
// Keyring helpers
// ---------------------------------------------------------------------------

const KEYRING_SERVICE: &str = "romm-cli";

/// Store a secret in the OS keyring under the `romm-cli` service name.
pub fn keyring_store(key: &str, value: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| anyhow!("keyring entry error: {e}"))?;
    entry
        .set_password(value)
        .map_err(|e| anyhow!("keyring set error: {e}"))
}

/// Retrieve a secret from the OS keyring, returning `None` if not found.
fn keyring_get(key: &str) -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key).ok()?;
    entry.get_password().ok()
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

/// Directory for user-level config (`romm-cli` under the OS config dir).
pub fn user_config_dir() -> Option<PathBuf> {
    #[cfg(test)]
    if let Ok(dir) = std::env::var("ROMM_TEST_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    dirs::config_dir().map(|d| d.join("romm-cli"))
}

/// Path to the user-level `.env` file (`.../romm-cli/.env`).
pub fn user_config_env_path() -> Option<PathBuf> {
    user_config_dir().map(|d| d.join(".env"))
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Load env vars from `./.env` in cwd, then from the user config file.
/// Later files only set variables not already set (env or earlier file), so a project `.env` overrides the same keys in the user file.
pub fn load_layered_env() {
    let _ = dotenvy::dotenv();
    if let Some(path) = user_config_env_path() {
        if path.is_file() {
            let _ = dotenvy::from_path(path);
        }
    }
}

/// Read an env var, falling back to the OS keyring if not set.
fn env_or_keyring(key: &str) -> Option<String> {
    std::env::var(key).ok().or_else(|| keyring_get(key))
}

pub fn load_config() -> Result<Config> {
    let base_url = std::env::var("API_BASE_URL").map_err(|_| {
        anyhow!(
            "API_BASE_URL is not set. Set it in the environment, a .env file, or run: romm-cli init"
        )
    })?;

    let username = std::env::var("API_USERNAME").ok();
    let password = env_or_keyring("API_PASSWORD");
    let token = env_or_keyring("API_TOKEN").or_else(|| env_or_keyring("API_KEY"));
    let api_key = env_or_keyring("API_KEY");
    let api_key_header = std::env::var("API_KEY_HEADER").ok();

    let auth = if let (Some(user), Some(pass)) = (username, password) {
        // Priority 1: Basic auth
        Some(AuthConfig::Basic {
            username: user,
            password: pass,
        })
    } else if let (Some(key), Some(header)) = (api_key, api_key_header) {
        // Priority 2: API key in custom header (when both set, prefer over bearer)
        if !is_placeholder(&key) {
            Some(AuthConfig::ApiKey { header, key })
        } else {
            None
        }
    } else if let Some(tok) = token {
        // Priority 3: Bearer token (skip placeholders)
        if !is_placeholder(&tok) {
            Some(AuthConfig::Bearer { token: tok })
        } else {
            None
        }
    } else {
        None
    };

    Ok(Config { base_url, auth })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_auth_env() {
        for key in [
            "API_BASE_URL",
            "API_USERNAME",
            "API_PASSWORD",
            "API_TOKEN",
            "API_KEY",
            "API_KEY_HEADER",
        ] {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn prefers_basic_auth_over_other_modes() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_USERNAME", "user");
        std::env::set_var("API_PASSWORD", "pass");
        std::env::set_var("API_TOKEN", "token");
        std::env::set_var("API_KEY", "apikey");
        std::env::set_var("API_KEY_HEADER", "X-Api-Key");

        let cfg = load_config().expect("config should load");
        match cfg.auth {
            Some(AuthConfig::Basic { username, password }) => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("expected basic auth"),
        }
    }

    #[test]
    fn uses_api_key_header_when_token_missing() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_KEY", "real-key");
        std::env::set_var("API_KEY_HEADER", "X-Api-Key");

        let cfg = load_config().expect("config should load");
        match cfg.auth {
            Some(AuthConfig::ApiKey { header, key }) => {
                assert_eq!(header, "X-Api-Key");
                assert_eq!(key, "real-key");
            }
            _ => panic!("expected api key auth"),
        }
    }

    #[test]
    fn ignores_placeholder_bearer_token() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::set_var("API_BASE_URL", "http://example.test");
        std::env::set_var("API_TOKEN", "your-bearer-token-here");

        let cfg = load_config().expect("config should load");
        assert!(cfg.auth.is_none(), "placeholder token should be ignored");
    }

    #[test]
    fn layered_env_applies_user_file_for_unset_keys() {
        let _guard = env_lock().lock().expect("env lock");
        clear_auth_env();
        std::env::remove_var("API_BASE_URL");

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!("romm-layered-{ts}"));
        std::fs::create_dir_all(&base).unwrap();
        let work = base.join("work");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            base.join(".env"),
            "API_BASE_URL=http://from-user-file.test\n",
        )
        .unwrap();

        std::env::set_var("ROMM_TEST_CONFIG_DIR", base.as_os_str());
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&work).unwrap();

        load_layered_env();
        let cfg = load_config().expect("load from user .env");
        assert_eq!(cfg.base_url, "http://from-user-file.test");

        std::env::set_current_dir(old_cwd).unwrap();
        std::env::remove_var("ROMM_TEST_CONFIG_DIR");
        std::env::remove_var("API_BASE_URL");
        let _ = std::fs::remove_dir_all(&base);
    }
}
