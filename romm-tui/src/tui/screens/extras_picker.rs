//! Extras types and helpers for the inline extras tab.

use anyhow::Result;

use romm_api::config::RomsLayoutConfig;
use romm_api::core::download::resolve_download_directory;
use romm_api::core::extras::{
    build_cover_target, build_manual_target, build_update_dlc_file_targets_for_rom,
    extras_root_dir, related_rom_download_target, DownloadTarget,
};
use romm_api::types::{Rom, RomFile};

/// What to download when this row is checked and confirmed.
#[derive(Debug, Clone)]
pub enum ExtrasTargetSeed {
    RelatedRom(Box<Rom>),
    InternalRomFile(RomFile),
    Cover,
    Manual,
}

#[derive(Debug, Clone)]
pub struct ExtrasPickerItem {
    pub label: String,
    pub sublabel: String,
    pub checked: bool,
    pub seed: ExtrasTargetSeed,
}

/// Build download targets from a list of extras items (used by inline extras tab).
pub fn build_selected_targets_from_items(
    items: &[ExtrasPickerItem],
    rom: &Rom,
    layout: &RomsLayoutConfig,
    configured_download_dir: Option<&str>,
) -> Result<Vec<DownloadTarget>> {
    let out = resolve_download_directory(configured_download_dir)?;
    let root = extras_root_dir(layout, &out, rom)?;
    let mut targets = Vec::new();
    let internal_targets = build_update_dlc_file_targets_for_rom(rom, layout, &out)?;

    for item in items {
        if !item.checked {
            continue;
        }
        match &item.seed {
            ExtrasTargetSeed::RelatedRom(other) => {
                targets.push(related_rom_download_target(rom, other, &root));
            }
            ExtrasTargetSeed::InternalRomFile(file) => {
                if let Some(t) = internal_targets
                    .iter()
                    .find(|t| {
                        t.source_url
                            .contains(&format!("/api/roms/{}/files/", file.id))
                            || t.source_url
                                .contains(&format!("/api/romsfiles/{}/", file.id))
                    })
                    .cloned()
                {
                    targets.push(t);
                }
            }
            ExtrasTargetSeed::Cover => {
                if let Some(t) = build_cover_target(rom, &root) {
                    targets.push(t);
                }
            }
            ExtrasTargetSeed::Manual => {
                if let Some(t) = build_manual_target(rom, &root) {
                    targets.push(t);
                }
            }
        }
    }

    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::screens::game_detail::{
        GameDetailPrevious, GameDetailScreen, COVER_PANEL_WIDTH_DEFAULT,
    };
    use crate::tui::screens::SearchScreen;
    use romm_api::core::download::DownloadJob;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn minimal_rom() -> Rom {
        Rom {
            id: 1,
            platform_id: 2,
            platform_slug: Some("nes".into()),
            platform_fs_slug: Some("NES".into()),
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: "game.zip".into(),
            fs_name_no_tags: "game".into(),
            fs_name_no_ext: "game".into(),
            fs_extension: "zip".into(),
            fs_path: "/game.zip".into(),
            fs_size_bytes: 1,
            name: "Game".into(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            has_manual: false,
            path_manual: None,
            url_manual: None,
            is_unidentified: false,
            is_identified: true,
            files: Vec::new(),
            ra_id: None,
            merged_ra_metadata: None,
        }
    }

    fn detail_with_extras() -> GameDetailScreen {
        let mut primary = minimal_rom();
        primary.url_cover = Some("https://x/c.png".into());
        primary.url_manual = Some("https://x/m.pdf".into());

        let other = Rom {
            id: 2,
            ..minimal_rom()
        };

        let prev = GameDetailPrevious::Search(SearchScreen::new());
        let downloads = Arc::new(Mutex::new(Vec::<DownloadJob>::new()));
        GameDetailScreen::new(
            primary,
            vec![other],
            prev,
            downloads,
            COVER_PANEL_WIDTH_DEFAULT,
        )
    }

    fn test_download_dir(label: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("romm-extras-{label}-{}-{ts}", std::process::id()))
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct TestDownloadEnv {
        _guard: MutexGuard<'static, ()>,
        dir: PathBuf,
        prev_roms_dir: Option<String>,
        prev_download_dir: Option<String>,
    }

    impl TestDownloadEnv {
        fn new(label: &str) -> Self {
            let guard = env_lock().lock().expect("env lock");
            let dir = test_download_dir(label);
            let prev_roms_dir = std::env::var("ROMM_ROMS_DIR").ok();
            let prev_download_dir = std::env::var("ROMM_DOWNLOAD_DIR").ok();
            std::env::set_var("ROMM_ROMS_DIR", &dir);
            std::env::remove_var("ROMM_DOWNLOAD_DIR");
            Self {
                _guard: guard,
                dir,
                prev_roms_dir,
                prev_download_dir,
            }
        }
    }

    impl Drop for TestDownloadEnv {
        fn drop(&mut self) {
            match &self.prev_roms_dir {
                Some(value) => std::env::set_var("ROMM_ROMS_DIR", value),
                None => std::env::remove_var("ROMM_ROMS_DIR"),
            }
            match &self.prev_download_dir {
                Some(value) => std::env::set_var("ROMM_DOWNLOAD_DIR", value),
                None => std::env::remove_var("ROMM_DOWNLOAD_DIR"),
            }
        }
    }

    #[test]
    fn extras_items_built_correctly() {
        let detail = detail_with_extras();
        assert_eq!(detail.extras_items.len(), 3);
        assert!(detail.extras_items.iter().all(|i| i.checked));
    }

    #[test]
    fn build_targets_empty_when_none_checked() {
        let mut detail = detail_with_extras();
        for i in &mut detail.extras_items {
            i.checked = false;
        }
        let env = TestDownloadEnv::new("empty");
        let dir = env.dir.clone();
        let targets = build_selected_targets_from_items(
            &detail.extras_items,
            &detail.rom,
            &RomsLayoutConfig::default(),
            Some("ignored"),
        )
        .unwrap();
        assert!(targets.is_empty());
        drop(env);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn build_targets_for_checked_items_only() {
        let mut detail = detail_with_extras();
        for i in &mut detail.extras_items {
            i.checked = false;
        }
        detail.extras_items[1].checked = true; // cover only

        let env = TestDownloadEnv::new("cover");
        let dir = env.dir.clone();
        let targets = build_selected_targets_from_items(
            &detail.extras_items,
            &detail.rom,
            &RomsLayoutConfig::default(),
            Some("ignored"),
        )
        .expect("targets");
        assert_eq!(targets.len(), 1);
        assert!(matches!(
            targets[0].kind,
            romm_api::core::extras::DownloadAssetKind::Cover
        ));
        drop(env);
        let _ = std::fs::remove_dir_all(dir);
    }
}
