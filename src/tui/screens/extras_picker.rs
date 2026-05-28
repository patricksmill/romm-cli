//! Full-screen picker for which extras to download (TUI).

use std::time::Duration;

use anyhow::Result;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use std::time::Instant;

use crate::config::ExtrasDefaults;
use crate::config::RomsLayoutConfig;
use crate::core::download::resolve_download_directory;
use crate::core::extras::{
    build_cover_target, build_manual_target, build_update_dlc_file_targets_for_rom,
    collect_update_dlc_files, extras_root_dir, related_rom_download_target, DownloadTarget,
};
use crate::tui::theme::RommStyles;
use crate::types::{Rom, RomFile};

use super::game_detail::GameDetailScreen;

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

/// Select extras then confirm to queue a composite download job.
pub struct ExtrasPickerScreen {
    pub rom: Rom,
    pub items: Vec<ExtrasPickerItem>,
    pub selected_index: usize,
    pub previous: Box<GameDetailScreen>,
    pub message: Option<String>,
    pub message_clear_at: Option<Instant>,
}

impl ExtrasPickerScreen {
    pub fn new(previous: Box<GameDetailScreen>, defaults: &ExtrasDefaults) -> Self {
        let rom = previous.rom.clone();
        let mut items = Vec::new();

        for other in &previous.other_files {
            items.push(ExtrasPickerItem {
                label: other.fs_name.clone(),
                sublabel: format!("Related ROM (id {})", other.id),
                checked: defaults.include_related_roms,
                seed: ExtrasTargetSeed::RelatedRom(Box::new(other.clone())),
            });
        }

        for file in collect_update_dlc_files(&rom) {
            let tag = match file.category {
                Some(crate::types::RomFileCategory::Update) => "Update",
                Some(crate::types::RomFileCategory::Dlc) => "DLC",
                _ => "ROM file",
            };
            items.push(ExtrasPickerItem {
                label: file.file_name.clone(),
                sublabel: format!("{tag} (file id {})", file.id),
                checked: defaults.include_related_roms,
                seed: ExtrasTargetSeed::InternalRomFile(file),
            });
        }

        if rom
            .url_cover
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some()
        {
            items.push(ExtrasPickerItem {
                label: "Cover image".to_string(),
                sublabel: "From url_cover".to_string(),
                checked: defaults.include_cover,
                seed: ExtrasTargetSeed::Cover,
            });
        }

        if rom
            .url_manual
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some()
        {
            items.push(ExtrasPickerItem {
                label: "Manual".to_string(),
                sublabel: "From url_manual".to_string(),
                checked: defaults.include_manual,
                seed: ExtrasTargetSeed::Manual,
            });
        }

        Self {
            rom,
            items,
            selected_index: 0,
            previous,
            message: None,
            message_clear_at: None,
        }
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|i| i.checked).count()
    }

    pub fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    pub fn toggle_current(&mut self) {
        if let Some(i) = self.items.get_mut(self.selected_index) {
            i.checked = !i.checked;
        }
    }

    pub fn toggle_all(&mut self) {
        let any_unchecked = self.items.iter().any(|i| !i.checked);
        for i in &mut self.items {
            i.checked = any_unchecked;
        }
    }

    pub fn show_message(&mut self, msg: impl Into<String>, ttl: Duration) {
        self.message = Some(msg.into());
        self.message_clear_at = Some(Instant::now() + ttl);
    }

    pub fn tick_message(&mut self) {
        if let Some(clear_at) = self.message_clear_at {
            if Instant::now() >= clear_at {
                self.message = None;
                self.message_clear_at = None;
            }
        }
    }

    /// Build download targets for checked rows. Fails if ROM dir is not configured.
    pub fn build_selected_targets(
        &self,
        layout: &RomsLayoutConfig,
        configured_download_dir: Option<&str>,
    ) -> Result<Vec<DownloadTarget>> {
        let out = resolve_download_directory(configured_download_dir)?;
        let root = extras_root_dir(layout, &out, &self.rom)?;
        let mut targets = Vec::new();
        let internal_targets = build_update_dlc_file_targets_for_rom(&self.rom, layout, &out)?;

        for item in &self.items {
            if !item.checked {
                continue;
            }
            match &item.seed {
                ExtrasTargetSeed::RelatedRom(other) => {
                    targets.push(related_rom_download_target(&self.rom, other, &root));
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
                    if let Some(t) = build_cover_target(&self.rom, &root) {
                        targets.push(t);
                    }
                }
                ExtrasTargetSeed::Manual => {
                    if let Some(t) = build_manual_target(&self.rom, &root) {
                        targets.push(t);
                    }
                }
            }
        }

        Ok(targets)
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, styles: &RommStyles) {
        self.tick_message();
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .direction(Direction::Vertical)
            .split(area);

        let title = format!("Extras — {}", self.rom.name);
        f.render_widget(
            Paragraph::new(title)
                .alignment(Alignment::Center)
                .style(styles.primary_text().add_modifier(Modifier::BOLD))
                .block(styles.panel_block_untitled()),
            chunks[0],
        );

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|it| {
                let mark = if it.checked { "[x]" } else { "[ ]" };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", mark), styles.label()),
                    Span::styled(&it.label, styles.primary_text()),
                    Span::raw(" — "),
                    Span::styled(&it.sublabel, styles.muted()),
                ]))
            })
            .collect();

        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        let list = List::new(list_items)
            .block(styles.panel_block("Items"))
            .highlight_style(styles.selection());

        f.render_stateful_widget(list, chunks[1], &mut state);

        let hint = self.message.as_deref().unwrap_or(
            "↑/↓: Navigate | Space: Toggle | a: Toggle all | Enter: Download selected | Esc: Back",
        );
        f.render_widget(
            Paragraph::new(hint)
                .block(styles.panel_block_untitled())
                .style(styles.muted()),
            chunks[2],
        );

        let footer = Paragraph::new("At least one item must be checked to start download.")
            .alignment(Alignment::Center)
            .style(styles.footer_hint())
            .block(styles.panel_block_untitled());
        f.render_widget(footer, chunks[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::download::DownloadJob;
    use crate::tui::screens::game_detail::{GameDetailPrevious, GameDetailScreen};
    use crate::tui::screens::SearchScreen;
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
        GameDetailScreen::new(primary, vec![other], prev, downloads)
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
    fn picker_seeds_checked_state_from_defaults() {
        let detail = detail_with_extras();
        let defaults = ExtrasDefaults {
            include_related_roms: true,
            include_cover: false,
            include_manual: true,
        };
        let picker = ExtrasPickerScreen::new(Box::new(detail), &defaults);
        assert_eq!(picker.items.len(), 3);
        assert!(picker.items[0].checked);
        assert!(!picker.items[1].checked);
        assert!(picker.items[2].checked);
    }

    #[test]
    fn picker_toggle_all_inverts_when_any_unchecked() {
        let detail = detail_with_extras();
        let defaults = ExtrasDefaults::default();
        let mut picker = ExtrasPickerScreen::new(Box::new(detail), &defaults);
        picker.items[0].checked = false;
        picker.toggle_all();
        assert!(picker.items.iter().all(|i| i.checked));
        picker.toggle_all();
        assert!(picker.items.iter().all(|i| !i.checked));
    }

    #[test]
    fn build_selected_targets_empty_when_none_checked() {
        let detail = detail_with_extras();
        let mut picker = ExtrasPickerScreen::new(Box::new(detail), &ExtrasDefaults::default());
        for i in &mut picker.items {
            i.checked = false;
        }
        let env = TestDownloadEnv::new("empty");
        let dir = env.dir.clone();
        let targets = picker
            .build_selected_targets(&RomsLayoutConfig::default(), Some("ignored"))
            .unwrap();
        assert!(targets.is_empty());
        drop(env);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn picker_emits_targets_for_checked_items_only() {
        let detail = detail_with_extras();
        let mut picker = ExtrasPickerScreen::new(Box::new(detail), &ExtrasDefaults::default());
        for i in &mut picker.items {
            i.checked = false;
        }
        picker.items[1].checked = true; // cover only

        let env = TestDownloadEnv::new("cover");
        let dir = env.dir.clone();
        let targets = picker
            .build_selected_targets(&RomsLayoutConfig::default(), Some("ignored"))
            .expect("targets");
        assert_eq!(targets.len(), 1);
        assert!(matches!(
            targets[0].kind,
            crate::core::extras::DownloadAssetKind::Cover
        ));
        drop(env);
        let _ = std::fs::remove_dir_all(dir);
    }
}
