//! Config construction and pairing exchange for the setup wizard.

use anyhow::{anyhow, Context, Result};

use crate::client::RommClient;
use crate::config::{
    default_theme_id, is_keyring_placeholder, load_config, normalize_romm_origin,
    persist_user_config, read_user_config_json_from_disk, AuthConfig, Config, RomsLayoutConfig,
};
use crate::core::download::validate_configured_download_directory;
use crate::endpoints::client_tokens::ExchangeClientToken;
use crate::tui::path_picker::{PathPicker, PathPickerMode};

use super::layout::extras_defaults_from_disk;
use super::types::{AuthKind, SetupWizard, Step};

impl SetupWizard {
    pub fn new() -> Self {
        let default_dl = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"))
            .join("romm-cli")
            .display()
            .to_string();
        Self {
            step: Step::Url,
            auth_kind: AuthKind::Pairing,
            auth_menu_selected: 0,
            url: "https://".to_string(),
            url_cursor: "https://".len(),
            download_picker: PathPicker::new(PathPickerMode::Directory, &default_dl),
            username: String::new(),
            user_cursor: 0,
            password: String::new(),
            bearer_token: String::new(),
            bearer_cursor: 0,
            api_header: String::new(),
            header_cursor: 0,
            api_key: String::new(),
            api_key_cursor: 0,
            pairing_code: String::new(),
            pairing_cursor: 0,
            reuse_keyring_password: false,
            reuse_keyring_bearer: false,
            reuse_keyring_api_key: false,
            testing: false,
            use_https: true,
            skip_custom_console_paths: false,
            error: None,
        }
    }

    pub fn new_auth_only(config: &Config) -> Self {
        let mut wizard = Self::new();
        wizard.step = Step::AuthMenu;
        wizard.url = config.base_url.clone();
        wizard
            .download_picker
            .set_path_text(config.download_dir.clone());
        wizard.use_https = config.use_https;
        wizard.skip_custom_console_paths = true;

        let disk = read_user_config_json_from_disk();

        match &config.auth {
            Some(AuthConfig::Basic { username, password }) => {
                wizard.auth_kind = AuthKind::Basic;
                wizard.auth_menu_selected = 1;
                wizard.username = username.clone();
                wizard.user_cursor = username.len();
                let disk_pass = disk
                    .as_ref()
                    .and_then(|c| c.auth.as_ref())
                    .and_then(|a| match a {
                        AuthConfig::Basic { password, .. } => Some(password.as_str()),
                        _ => None,
                    });
                if disk_pass.is_some_and(is_keyring_placeholder) {
                    wizard.password = String::new();
                    wizard.reuse_keyring_password = true;
                } else {
                    wizard.password = password.clone();
                }
            }
            Some(AuthConfig::Bearer { token }) => {
                wizard.auth_kind = AuthKind::Bearer;
                wizard.auth_menu_selected = 2;
                let disk_tok = disk
                    .as_ref()
                    .and_then(|c| c.auth.as_ref())
                    .and_then(|a| match a {
                        AuthConfig::Bearer { token } => Some(token.as_str()),
                        _ => None,
                    });
                if disk_tok.is_some_and(is_keyring_placeholder) {
                    wizard.bearer_token = String::new();
                    wizard.bearer_cursor = 0;
                    wizard.reuse_keyring_bearer = true;
                } else {
                    wizard.bearer_token = token.clone();
                    wizard.bearer_cursor = token.len();
                }
            }
            Some(AuthConfig::ApiKey { header, key }) => {
                wizard.auth_kind = AuthKind::ApiKey;
                wizard.auth_menu_selected = 3;
                wizard.api_header = header.clone();
                wizard.header_cursor = header.len();
                let disk_key = disk
                    .as_ref()
                    .and_then(|c| c.auth.as_ref())
                    .and_then(|a| match a {
                        AuthConfig::ApiKey { key, .. } => Some(key.as_str()),
                        _ => None,
                    });
                if disk_key.is_some_and(is_keyring_placeholder) {
                    wizard.api_key = String::new();
                    wizard.api_key_cursor = 0;
                    wizard.reuse_keyring_api_key = true;
                } else {
                    wizard.api_key = key.clone();
                    wizard.api_key_cursor = key.len();
                }
            }
            None => {
                wizard.auth_kind = AuthKind::Pairing;
                wizard.auth_menu_selected = 0;
            }
        }
        wizard
    }

    pub(crate) fn auth_labels() -> [&'static str; 4] {
        [
            "Pair with Web UI (8-character code) (Recommended)",
            "Username + password",
            "API Token",
            "API key in custom header",
        ]
    }

    pub(crate) fn auth_kind_from_index(i: usize) -> AuthKind {
        match i {
            0 => AuthKind::Pairing,
            1 => AuthKind::Basic,
            2 => AuthKind::Bearer,
            _ => AuthKind::ApiKey,
        }
    }

    fn roms_layout_from_wizard(&self) -> RomsLayoutConfig {
        read_user_config_json_from_disk()
            .map(|c| c.roms_layout)
            .unwrap_or_default()
    }

    /// Build config after exchanging a Web UI pairing code (unauthenticated POST).
    pub(crate) async fn pairing_config_from_exchange(&self, verbose: bool) -> Result<Config> {
        let base_url = normalize_romm_origin(self.url.trim());
        if base_url.is_empty() {
            return Err(anyhow!("Server URL cannot be empty"));
        }
        let code = self.pairing_code.trim().to_string();
        if code.is_empty() {
            return Err(anyhow!("Pairing code cannot be empty"));
        }
        let download_dir =
            validate_configured_download_directory(self.download_picker.path_trimmed().trim())?
                .display()
                .to_string();
        let temp_config = Config {
            base_url: base_url.clone(),
            download_dir: download_dir.clone(),
            use_https: self.use_https,
            auth: None,
            extras_defaults: extras_defaults_from_disk(),
            save_sync: read_user_config_json_from_disk()
                .map(|c| c.save_sync)
                .unwrap_or_default(),
            roms_layout: self.roms_layout_from_wizard(),
            theme: read_user_config_json_from_disk()
                .map(|c| c.theme)
                .unwrap_or_else(default_theme_id),
        };
        let client = RommClient::new(&temp_config, verbose)?;
        let response = client
            .call(&ExchangeClientToken { code })
            .await
            .context("failed to exchange pairing code")?;
        Ok(Config {
            base_url,
            download_dir,
            use_https: self.use_https,
            auth: Some(AuthConfig::Bearer {
                token: response.raw_token,
            }),
            extras_defaults: extras_defaults_from_disk(),
            save_sync: read_user_config_json_from_disk()
                .map(|c| c.save_sync)
                .unwrap_or_default(),
            roms_layout: self.roms_layout_from_wizard(),
            theme: read_user_config_json_from_disk()
                .map(|c| c.theme)
                .unwrap_or_else(default_theme_id),
        })
    }

    fn build_config(&self) -> Result<Config> {
        let base_url = normalize_romm_origin(self.url.trim());
        if base_url.is_empty() {
            return Err(anyhow!("Server URL cannot be empty"));
        }
        let download_dir =
            validate_configured_download_directory(self.download_picker.path_trimmed().trim())?
                .display()
                .to_string();
        let auth: Option<AuthConfig> = match self.auth_kind {
            AuthKind::Basic => {
                let u = self.username.trim();
                if u.is_empty() {
                    return Err(anyhow!("Username cannot be empty"));
                }
                let password = if self.password.is_empty() && self.reuse_keyring_password {
                    crate::config::keyring_get("API_PASSWORD").ok_or_else(|| {
                        anyhow!("Password not in OS keyring; enter a password or run romm-cli init")
                    })?
                } else if self.password.is_empty() {
                    return Err(anyhow!("Password cannot be empty"));
                } else {
                    self.password.clone()
                };
                Some(AuthConfig::Basic {
                    username: u.to_string(),
                    password,
                })
            }
            AuthKind::Bearer => {
                let token = if self.bearer_token.trim().is_empty() && self.reuse_keyring_bearer {
                    crate::config::keyring_get("API_TOKEN").ok_or_else(|| {
                        anyhow!("API token not in OS keyring; enter a token or run romm-cli init")
                    })?
                } else if self.bearer_token.trim().is_empty() {
                    return Err(anyhow!("Bearer token cannot be empty"));
                } else {
                    self.bearer_token.trim().to_string()
                };
                Some(AuthConfig::Bearer { token })
            }
            AuthKind::ApiKey => {
                let h = self.api_header.trim();
                if h.is_empty() {
                    return Err(anyhow!("Header name cannot be empty"));
                }
                let key = if self.api_key.is_empty() && self.reuse_keyring_api_key {
                    crate::config::keyring_get("API_KEY").ok_or_else(|| {
                        anyhow!("API key not in OS keyring; enter a key or run romm-cli init")
                    })?
                } else if self.api_key.is_empty() {
                    return Err(anyhow!("API key cannot be empty"));
                } else {
                    self.api_key.clone()
                };
                Some(AuthConfig::ApiKey {
                    header: h.to_string(),
                    key,
                })
            }
            AuthKind::Pairing => {
                return Err(anyhow!(
                    "Pairing auth is applied when connecting; use the pairing code step and connect"
                ));
            }
        };
        Ok(Config {
            base_url,
            download_dir,
            use_https: self.use_https,
            auth,
            extras_defaults: extras_defaults_from_disk(),
            save_sync: read_user_config_json_from_disk()
                .map(|c| c.save_sync)
                .unwrap_or_default(),
            roms_layout: self.roms_layout_from_wizard(),
            theme: read_user_config_json_from_disk()
                .map(|c| c.theme)
                .unwrap_or_else(default_theme_id),
        })
    }

    pub async fn try_connect_and_persist(&mut self, verbose: bool) -> Result<Config> {
        let cfg = if self.auth_kind == AuthKind::Pairing {
            self.pairing_config_from_exchange(verbose).await?
        } else {
            self.build_config()?
        };
        let client = RommClient::new(&cfg, verbose)?;
        client.fetch_openapi_json().await?;
        persist_user_config(&cfg)?;
        load_config()
    }
}
