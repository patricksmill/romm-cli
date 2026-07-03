use std::sync::{Arc, Mutex};

use ratatui_image::picker::ProtocolType;

use super::cover::detect_cover_protocol_from_env;
use super::saves::save_lines;
use super::types::{
    CoverState, DetailTab, GameDetailPrevious, GameDetailScreen, SaveListState,
    COVER_PANEL_WIDTH_DEFAULT,
};
use crate::tui::screens::SearchScreen;
use romm_api::types::SaveMetadata;

#[test]
fn detect_cover_protocol_prefers_kitty_hint() {
    let protocol = detect_cover_protocol_from_env(
        Some("iTerm.app".into()),
        Some("xterm-256color".into()),
        Some("123".into()),
    );
    assert_eq!(protocol, Some(ProtocolType::Kitty));
}

#[test]
fn detect_cover_protocol_supports_sixel_term() {
    let protocol =
        detect_cover_protocol_from_env(None, Some("xterm+sixel".into()), Some(String::new()));
    assert_eq!(protocol, Some(ProtocolType::Sixel));
}

#[test]
fn missing_protocol_still_requests_cover_load() {
    let rom = romm_api::types::Rom {
        id: 5,
        platform_id: 1,
        platform_slug: None,
        platform_fs_slug: None,
        platform_custom_name: None,
        platform_display_name: None,
        fs_name: "game.zip".to_string(),
        fs_name_no_tags: "game".to_string(),
        fs_name_no_ext: "game".to_string(),
        fs_extension: "zip".to_string(),
        fs_path: "/game.zip".to_string(),
        fs_size_bytes: 10,
        name: "game".to_string(),
        slug: None,
        summary: None,
        path_cover_small: None,
        path_cover_large: None,
        url_cover: Some("http://example.com/cover.png".to_string()),
        has_manual: false,
        path_manual: None,
        url_manual: None,
        is_unidentified: false,
        is_identified: true,
        files: Vec::new(),
        ra_id: None,
        merged_ra_metadata: None,
    };
    let previous = GameDetailPrevious::Search(SearchScreen::new());
    let downloads = Arc::new(Mutex::new(Vec::new()));
    let mut detail = GameDetailScreen::new(
        rom,
        Vec::new(),
        previous,
        downloads,
        COVER_PANEL_WIDTH_DEFAULT,
    );
    detail.cover_protocol = None;
    assert!(detail.should_request_cover_load());
    detail.set_cover_loading();
    assert_eq!(detail.cover_state, CoverState::Loading);
}

#[test]
fn has_any_extras_false_when_no_assets() {
    let rom = test_rom(1, None);
    let detail = new_detail(rom);
    assert!(!detail.has_any_extras());
}

#[test]
fn footer_info_tab_shows_match_for_unidentified() {
    let mut rom = test_rom(1, None);
    rom.is_identified = false;
    let detail = new_detail(rom);
    assert_eq!(detail.active_tab, DetailTab::Info);
    let entries = detail.footer_help_entries();
    assert!(entries
        .iter()
        .any(|entry| entry.key == "m" && entry.label == "Match"));
    assert!(entries
        .iter()
        .any(|entry| entry.key == "Ctrl+←/→" && entry.label == "Resize cover"));
    assert!(entries
        .iter()
        .any(|entry| entry.key == "1-5" && entry.label == "Tabs"));
}

#[test]
fn footer_info_tab_shows_unmatch_for_identified() {
    let detail = new_detail(test_rom(1, None));
    assert_eq!(detail.active_tab, DetailTab::Info);
    let entries = detail.footer_help_entries();
    assert!(entries
        .iter()
        .any(|entry| entry.key == "m" && entry.label == "Unmatch"));
}

#[test]
fn footer_extras_tab_hints() {
    let mut detail = new_detail(test_rom(1, None));
    detail.select_tab(DetailTab::Extras);
    let entries = detail.footer_help_entries();
    assert!(entries
        .iter()
        .any(|entry| entry.key == "Space" && entry.label == "Toggle"));
    assert!(entries
        .iter()
        .any(|entry| entry.key == "Enter" && entry.label == "Download"));
}

#[test]
fn footer_technical_tab_has_match_unmatch() {
    let detail = new_detail(test_rom(1, None));
    let mut d = detail;
    d.select_tab(DetailTab::Technical);
    let entries = d.footer_help_entries();
    assert!(entries
        .iter()
        .any(|entry| entry.key == "m" && entry.label == "Unmatch"));
}

#[test]
fn footer_saves_tab_shows_upload_download() {
    let mut detail = new_detail(test_rom(1, None));
    detail.select_tab(DetailTab::Saves);
    let entries = detail.footer_help_entries();
    assert!(entries
        .iter()
        .any(|entry| entry.key == "u" && entry.label == "Upload save"));
    assert!(entries
        .iter()
        .any(|entry| entry.key == "D" && entry.label == "Download save"));
}

#[test]
fn tab_switching_changes_active_tab() {
    let mut detail = new_detail(test_rom(1, None));
    assert_eq!(detail.active_tab, DetailTab::Info);
    detail.select_tab(DetailTab::Extras);
    assert_eq!(detail.active_tab, DetailTab::Extras);
    detail.select_tab(DetailTab::Saves);
    assert_eq!(detail.active_tab, DetailTab::Saves);
    detail.select_tab(DetailTab::Achievements);
    assert_eq!(detail.active_tab, DetailTab::Achievements);
    detail.select_tab(DetailTab::Technical);
    assert_eq!(detail.active_tab, DetailTab::Technical);
    detail.select_tab(DetailTab::Info);
    assert_eq!(detail.active_tab, DetailTab::Info);
}

#[test]
fn cover_panel_width_defaults_to_forty_two() {
    let detail = new_detail(test_rom(1, None));
    assert_eq!(detail.cover_panel_width, 42);
}

#[test]
fn adjust_cover_panel_width_clamps_at_bounds() {
    let mut detail = new_detail(test_rom(1, None));
    detail.adjust_cover_panel_width(-100);
    assert_eq!(detail.cover_panel_width, 20);
    detail.adjust_cover_panel_width(100);
    assert_eq!(detail.cover_panel_width, 60);
}

#[test]
fn cover_state_transitions_to_ready_and_error() {
    let mut detail = new_detail(test_rom(1, Some("http://example.com/cover.png".into())));
    detail.set_cover_loading();
    assert_eq!(detail.cover_state, CoverState::Loading);

    detail.apply_cover_image(image::DynamicImage::new_rgba8(4, 4));
    assert_eq!(detail.cover_state, CoverState::Ready);
    assert!(detail.cover_image.is_some());

    detail.apply_cover_error("oops".to_string());
    assert_eq!(detail.cover_state, CoverState::Failed("oops".to_string()));
}

#[test]
fn save_list_formatting_handles_states() {
    assert!(save_lines(&SaveListState::Loading, 0)[0]
        .to_string()
        .contains("Loading"));
    assert!(save_lines(&SaveListState::Failed("boom".into()), 0)[0]
        .to_string()
        .contains("Error"));
    assert!(save_lines(&SaveListState::Loaded(vec![]), 0)[0]
        .to_string()
        .contains("No remote saves"));

    let rows = vec![SaveMetadata {
        id: 7,
        file_name: "game.sav".into(),
        emulator: Some("duckstation".into()),
        slot: Some("1".into()),
        updated_at: Some("2026-05-12T00:00:00Z".into()),
        hash: Some("abcdef1234567890".into()),
        size_bytes: Some(1024),
        device_id: Some("dev1".into()),
        device_name: None,
        screenshot: None,
    }];
    let line = save_lines(&SaveListState::Loaded(rows), 0)[0].to_string();
    assert!(line.contains("> game.sav"));
    assert!(line.contains("emu=duckstation"));
    assert!(line.contains("slot=1"));
}

fn test_rom(id: u64, url_cover: Option<String>) -> romm_api::types::Rom {
    romm_api::types::Rom {
        id,
        platform_id: 1,
        platform_slug: None,
        platform_fs_slug: None,
        platform_custom_name: None,
        platform_display_name: None,
        fs_name: "game.zip".to_string(),
        fs_name_no_tags: "game".to_string(),
        fs_name_no_ext: "game".to_string(),
        fs_extension: "zip".to_string(),
        fs_path: "/game.zip".to_string(),
        fs_size_bytes: 10,
        name: "game".to_string(),
        slug: None,
        summary: None,
        path_cover_small: None,
        path_cover_large: None,
        url_cover,
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

#[test]
fn achievement_selection_nav_clamps() {
    let mut detail = new_detail(test_rom(1, None));
    use super::types::AchievementListState;
    detail.achievements_state = AchievementListState::Loaded {
        rows: vec![
            romm_api::types::AchievementRow {
                title: "A".into(),
                description: Some("Desc A".into()),
                points: Some(5),
                earned: false,
                earned_at: None,
            },
            romm_api::types::AchievementRow {
                title: "B".into(),
                description: None,
                points: Some(10),
                earned: true,
                earned_at: Some("2024-01-01".into()),
            },
        ],
        summary: (1, 2),
    };
    assert_eq!(detail.selected_achievement_index, 0);
    detail.achievement_selection_next();
    assert_eq!(detail.selected_achievement_index, 1);
    detail.achievement_selection_next();
    assert_eq!(detail.selected_achievement_index, 1);
    detail.achievement_selection_previous();
    assert_eq!(detail.selected_achievement_index, 0);
    detail.achievement_selection_previous();
    assert_eq!(detail.selected_achievement_index, 0);

    let sel = detail.selected_achievement().unwrap();
    assert_eq!(sel.title, "A");
    assert_eq!(sel.description.as_deref(), Some("Desc A"));
}

#[test]
fn save_screenshot_state_transitions() {
    let mut detail = new_detail(test_rom(1, None));
    assert_eq!(detail.save_screenshot_state, CoverState::Idle);
    assert!(detail.save_screenshot_image.is_none());

    detail.apply_save_screenshot_image(image::DynamicImage::new_rgba8(4, 4));
    assert_eq!(detail.save_screenshot_state, CoverState::Ready);
    assert!(detail.save_screenshot_image.is_some());

    detail.apply_save_screenshot_error("bad".to_string());
    assert_eq!(
        detail.save_screenshot_state,
        CoverState::Failed("bad".to_string())
    );
    assert!(detail.save_screenshot_image.is_none());
}

fn new_detail(rom: romm_api::types::Rom) -> GameDetailScreen {
    GameDetailScreen::new(
        rom,
        Vec::new(),
        GameDetailPrevious::Search(SearchScreen::new()),
        Arc::new(Mutex::new(Vec::new())),
        COVER_PANEL_WIDTH_DEFAULT,
    )
}
