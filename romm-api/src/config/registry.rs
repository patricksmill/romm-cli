//! Config key registry: dotted-path parse/set and env var mapping (approach D).

use serde::Serialize;

use crate::error::ConfigError;

use super::Config;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigSource {
    Default,
    File,
    Env(String),
    Keyring,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SourcedValue<T> {
    pub value: T,
    pub source: ConfigSource,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConfigField<T> {
    pub value: T,
    pub source: ConfigSource,
}

impl<T: Default> Default for ConfigField<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            source: ConfigSource::Default,
        }
    }
}

/// Per-field configuration source attribution (populated by [`super::load_config_with_sources`]).
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct ConfigSources {
    pub base_url: ConfigField<String>,
    pub download_dir: ConfigField<String>,
    pub use_https: ConfigField<bool>,
    pub theme: ConfigField<String>,
    pub extras_include_related_roms: ConfigField<bool>,
    pub extras_include_cover: ConfigField<bool>,
    pub extras_include_manual: ConfigField<bool>,
    pub save_sync_save_dir: ConfigField<Option<String>>,
    pub save_sync_device_id: ConfigField<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigKey {
    BaseUrl,
    DownloadDir,
    UseHttps,
    Theme,
    ExtrasIncludeRelatedRoms,
    ExtrasIncludeCover,
    ExtrasIncludeManual,
    SaveSyncSaveDir,
    SaveSyncDeviceId,
    SaveSyncPlatformDir(u64),
    RomsPlatformDir(u64),
    TuiLayoutLibraryLeftPanelPercent,
    TuiLayoutGameDetailCoverPanelWidth,
}

impl ConfigKey {
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        match s {
            "base_url" => Ok(Self::BaseUrl),
            "download_dir" => Ok(Self::DownloadDir),
            "use_https" => Ok(Self::UseHttps),
            "theme" => Ok(Self::Theme),
            "extras_defaults.include_related_roms" => Ok(Self::ExtrasIncludeRelatedRoms),
            "extras_defaults.include_cover" => Ok(Self::ExtrasIncludeCover),
            "extras_defaults.include_manual" => Ok(Self::ExtrasIncludeManual),
            "save_sync.save_dir" => Ok(Self::SaveSyncSaveDir),
            "save_sync.device_id" => Ok(Self::SaveSyncDeviceId),
            "tui_layout.library_left_panel_percent" => Ok(Self::TuiLayoutLibraryLeftPanelPercent),
            "tui_layout.game_detail_cover_panel_width" => {
                Ok(Self::TuiLayoutGameDetailCoverPanelWidth)
            }
            key if key.starts_with("save_sync.platform_dirs.") => {
                let id = key
                    .strip_prefix("save_sync.platform_dirs.")
                    .ok_or_else(|| ConfigError::Other(format!("unknown config key: {s}")))?;
                let platform_id = id
                    .parse::<u64>()
                    .map_err(|_| ConfigError::Other(format!("invalid platform id in key: {s}")))?;
                Ok(Self::SaveSyncPlatformDir(platform_id))
            }
            key if key.starts_with("roms_layout.platform_dirs.") => {
                let id = key
                    .strip_prefix("roms_layout.platform_dirs.")
                    .ok_or_else(|| ConfigError::Other(format!("unknown config key: {s}")))?;
                let platform_id = id
                    .parse::<u64>()
                    .map_err(|_| ConfigError::Other(format!("invalid platform id in key: {s}")))?;
                Ok(Self::RomsPlatformDir(platform_id))
            }
            _ => Err(ConfigError::Other(format!("unknown config key: {s}"))),
        }
    }

    pub fn env_var(&self) -> Option<&'static str> {
        match self {
            Self::BaseUrl => Some("API_BASE_URL"),
            Self::DownloadDir => Some("ROMM_ROMS_DIR"),
            Self::UseHttps => Some("API_USE_HTTPS"),
            Self::Theme => Some("ROMM_THEME"),
            Self::ExtrasIncludeRelatedRoms => Some("ROMM_EXTRAS_INCLUDE_RELATED_ROMS"),
            Self::ExtrasIncludeCover => Some("ROMM_EXTRAS_INCLUDE_COVER"),
            Self::ExtrasIncludeManual => Some("ROMM_EXTRAS_INCLUDE_MANUAL"),
            Self::SaveSyncSaveDir => Some("ROMM_SAVE_SYNC_SAVE_DIR"),
            Self::SaveSyncDeviceId => Some("ROMM_SAVE_SYNC_DEVICE_ID"),
            Self::SaveSyncPlatformDir(_) | Self::RomsPlatformDir(_) => None,
            Self::TuiLayoutLibraryLeftPanelPercent | Self::TuiLayoutGameDetailCoverPanelWidth => {
                None
            }
        }
    }
}

/// Returns the primary environment variable for a dotted config key, if any.
pub fn env_var_for_key(key: &str) -> Option<&'static str> {
    ConfigKey::parse(key).ok().and_then(|k| k.env_var())
}

fn parse_bool(label: &str, raw: &str) -> Result<bool, ConfigError> {
    let t = raw.trim().to_ascii_lowercase();
    match t.as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(ConfigError::Other(format!(
            "Invalid boolean for {label}: {raw:?} (use true or false)"
        ))),
    }
}

/// Patch one field on an in-memory [`Config`] via a dotted key path.
pub fn set_config_key(config: &mut Config, key: &str, value: &str) -> Result<(), ConfigError> {
    match ConfigKey::parse(key)? {
        ConfigKey::BaseUrl => config.base_url = value.to_string(),
        ConfigKey::DownloadDir => config.download_dir = value.to_string(),
        ConfigKey::UseHttps => config.use_https = parse_bool(key, value)?,
        ConfigKey::Theme => config.theme = value.to_string(),
        ConfigKey::ExtrasIncludeRelatedRoms => {
            config.extras_defaults.include_related_roms = parse_bool(key, value)?;
        }
        ConfigKey::ExtrasIncludeCover => {
            config.extras_defaults.include_cover = parse_bool(key, value)?;
        }
        ConfigKey::ExtrasIncludeManual => {
            config.extras_defaults.include_manual = parse_bool(key, value)?;
        }
        ConfigKey::SaveSyncSaveDir => {
            config.save_sync.save_dir = if value.trim().is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        }
        ConfigKey::SaveSyncDeviceId => {
            config.save_sync.device_id = if value.trim().is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        }
        ConfigKey::SaveSyncPlatformDir(id) => {
            config.save_sync.platform_dirs.insert(id, value.to_string());
        }
        ConfigKey::RomsPlatformDir(id) => {
            config.roms_layout.platform_dirs.insert(id, value.to_string());
        }
        ConfigKey::TuiLayoutLibraryLeftPanelPercent => {
            config.tui_layout.library_left_panel_percent = value.parse().map_err(|_| {
                ConfigError::Other(format!("invalid u16 for {key}: {value}"))
            })?;
            config.tui_layout = config.tui_layout.clone().normalized();
        }
        ConfigKey::TuiLayoutGameDetailCoverPanelWidth => {
            config.tui_layout.game_detail_cover_panel_width = value.parse().map_err(|_| {
                ConfigError::Other(format!("invalid u16 for {key}: {value}"))
            })?;
            config.tui_layout = config.tui_layout.clone().normalized();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ExtrasDefaults, RomsLayoutConfig, SaveSyncConfig, TuiLayoutConfig, default_theme_id,
    };

    #[test]
    fn parse_dotted_keys() {
        assert_eq!(
            ConfigKey::parse("save_sync.device_id").unwrap(),
            ConfigKey::SaveSyncDeviceId
        );
        assert_eq!(
            ConfigKey::parse("roms_layout.platform_dirs.7").unwrap(),
            ConfigKey::RomsPlatformDir(7)
        );
    }

    #[test]
    fn set_and_get_extras_bool() {
        let mut cfg = minimal_config();
        set_config_key(&mut cfg, "extras_defaults.include_cover", "false").unwrap();
        assert!(!cfg.extras_defaults.include_cover);
    }

    fn minimal_config() -> Config {
        Config {
            base_url: "http://localhost".into(),
            download_dir: "/tmp/roms".into(),
            use_https: false,
            auth: None,
            extras_defaults: ExtrasDefaults::default(),
            save_sync: SaveSyncConfig::default(),
            roms_layout: RomsLayoutConfig::default(),
            theme: default_theme_id(),
            tui_layout: TuiLayoutConfig::default(),
        }
    }
}
