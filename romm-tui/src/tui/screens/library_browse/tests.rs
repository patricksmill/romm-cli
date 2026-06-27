use super::LibraryBrowseScreen;
use super::LibrarySearchMode;
use super::LibraryViewMode;
use super::LEFT_PANEL_PERCENT_DEFAULT;
use romm_api::core::utils;
use romm_api::types::{Platform, Rom, RomList};
use serde_json::json;

fn rom(id: u64, name: &str, fs_name: &str) -> Rom {
    Rom {
        id,
        platform_id: 1,
        platform_slug: None,
        platform_fs_slug: None,
        platform_custom_name: None,
        platform_display_name: None,
        fs_name: fs_name.to_string(),
        fs_name_no_tags: name.to_string(),
        fs_name_no_ext: name.to_string(),
        fs_extension: "zip".to_string(),
        fs_path: format!("/{id}.zip"),
        fs_size_bytes: 1,
        name: name.to_string(),
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

fn platform(id: u64, name: &str, rom_count: u64) -> Platform {
    serde_json::from_value(json!({
        "id": id,
        "slug": format!("p{id}"),
        "fs_slug": format!("p{id}"),
        "rom_count": rom_count,
        "name": name,
        "igdb_slug": null,
        "moby_slug": null,
        "hltb_slug": null,
        "custom_name": null,
        "igdb_id": null,
        "sgdb_id": null,
        "moby_id": null,
        "launchbox_id": null,
        "ss_id": null,
        "ra_id": null,
        "hasheous_id": null,
        "tgdb_id": null,
        "flashpoint_id": null,
        "category": null,
        "generation": null,
        "family_name": null,
        "family_slug": null,
        "url": null,
        "url_logo": null,
        "firmware": [],
        "aspect_ratio": null,
        "created_at": "",
        "updated_at": "",
        "fs_size_bytes": 0,
        "is_unidentified": false,
        "is_identified": true,
        "missing_from_fs": false,
        "display_name": null
    }))
    .expect("valid platform fixture")
}

#[test]
fn list_filter_narrows_visible_rows() {
    let s = LibraryBrowseScreen::new(
        vec![
            platform(1, "Nintendo SNES", 10),
            platform(2, "Sega Genesis", 5),
            platform(3, "Sony PlayStation", 20),
        ],
        vec![],
        LEFT_PANEL_PERCENT_DEFAULT,
    );
    let mut s = s;
    s.enter_list_search(LibrarySearchMode::Filter);
    for c in "snes".chars() {
        s.add_list_search_char(c);
    }
    s.list_search.mode = None;
    s.list_search.filter_browsing = true;
    assert_eq!(s.list_len(), 1);
    assert_eq!(
        s.selected_list_source_index(),
        Some(0),
        "only SNES should match"
    );
}

#[test]
fn get_selected_group_clamps_stale_index_after_filter() {
    let mut s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    let items = vec![
        rom(1, "alpha", "a.zip"),
        rom(2, "alphabet", "ab.zip"),
        rom(3, "beta", "b.zip"),
    ];
    s.rom_groups = Some(utils::group_roms_by_name(&items));
    s.view_mode = LibraryViewMode::Roms;
    s.enter_rom_search(LibrarySearchMode::Filter);
    for c in "alp".chars() {
        s.add_rom_search_char(c);
    }
    s.rom_search.mode = None;
    s.rom_search.filter_browsing = true;
    s.rom_selected = 99;
    let (primary, _) = s
        .get_selected_group()
        .expect("clamped index should yield a group");
    assert_eq!(primary.name, "alpha");
}

#[test]
fn rom_next_wraps_within_filtered_list_when_filter_browsing() {
    let mut s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    let items = vec![
        rom(1, "alpha", "a.zip"),
        rom(2, "alphabet", "ab.zip"),
        rom(3, "beta", "b.zip"),
    ];
    s.rom_groups = Some(utils::group_roms_by_name(&items));
    s.view_mode = LibraryViewMode::Roms;
    s.enter_rom_search(LibrarySearchMode::Filter);
    for c in "alp".chars() {
        s.add_rom_search_char(c);
    }
    s.rom_search.mode = None;
    s.rom_search.filter_browsing = true;
    assert_eq!(s.rom_selected, 0);
    s.rom_next();
    assert_eq!(s.rom_selected, 1);
    s.rom_next();
    assert_eq!(s.rom_selected, 0);
}

#[test]
fn get_selected_group_clamps_stale_index() {
    let mut s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    let items = vec![
        rom(1, "alpha", "a.zip"),
        rom(2, "alphabet", "ab.zip"),
        rom(3, "beta", "b.zip"),
    ];
    s.rom_groups = Some(utils::group_roms_by_name(&items));
    s.view_mode = LibraryViewMode::Roms;
    s.rom_selected = 99;
    let (primary, _) = s
        .get_selected_group()
        .expect("clamped index should yield a group");
    assert_eq!(primary.name, "alpha");
}

#[test]
fn rom_next_wraps_within_list() {
    let mut s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    let items = vec![
        rom(1, "alpha", "a.zip"),
        rom(2, "alphabet", "ab.zip"),
        rom(3, "beta", "b.zip"),
    ];
    s.rom_groups = Some(utils::group_roms_by_name(&items));
    s.view_mode = LibraryViewMode::Roms;
    assert_eq!(s.rom_selected, 0);
    s.rom_next();
    assert_eq!(s.rom_selected, 1);
    s.rom_next();
    assert_eq!(s.rom_selected, 2);
    s.rom_next();
    assert_eq!(s.rom_selected, 0);
}

#[test]
fn zero_rom_platform_builds_no_rom_request() {
    let s = LibraryBrowseScreen::new(
        vec![platform(1, "Empty", 0)],
        vec![],
        LEFT_PANEL_PERCENT_DEFAULT,
    );
    assert!(
        s.get_roms_request_platform().is_none(),
        "zero-rom platform should not produce ROM API request"
    );
}

#[test]
fn back_to_list_retains_current_rom_state() {
    let mut s = LibraryBrowseScreen::new(
        vec![platform(1, "SNES", 12)],
        vec![],
        LEFT_PANEL_PERCENT_DEFAULT,
    );
    let items = vec![rom(1, "alpha", "a.zip")];
    let rom_list = RomList {
        total: 1,
        limit: 1,
        offset: 0,
        items: items.clone(),
    };
    s.view_mode = LibraryViewMode::Roms;
    s.roms = Some(rom_list);
    s.rom_groups = Some(utils::group_roms_by_name(&items));
    s.set_rom_loading(true);
    s.back_to_list();
    assert_eq!(s.view_mode, LibraryViewMode::List);
    assert!(
        s.roms.is_some(),
        "back navigation should keep loaded ROM list"
    );
    assert!(
        s.rom_groups.is_some(),
        "back navigation should keep grouped ROM rows"
    );
    assert!(
        s.rom_loading,
        "back navigation should preserve in-flight loading state"
    );
}

#[test]
fn empty_state_message_shows_loading_only_when_loading_flag_is_true() {
    let mut s = LibraryBrowseScreen::new(
        vec![platform(1, "SNES", 12)],
        vec![],
        LEFT_PANEL_PERCENT_DEFAULT,
    );
    s.clear_roms();
    s.set_rom_loading(false);
    assert_eq!(
        s.empty_rom_state_message(),
        "Select a console or collection and press Enter to load ROMs"
    );
    s.set_rom_loading(true);
    assert_eq!(s.empty_rom_state_message(), "Loading games...");
}

#[test]
fn left_panel_percent_defaults_to_thirty() {
    let s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    assert_eq!(s.left_panel_percent, 30);
}

#[test]
fn adjust_left_panel_percent_clamps_at_bounds() {
    let mut s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    s.adjust_left_panel_percent(-100);
    assert_eq!(s.left_panel_percent, 15);
    s.adjust_left_panel_percent(100);
    assert_eq!(s.left_panel_percent, 50);
}

#[test]
fn rom_row_not_highlighted_while_browsing_consoles() {
    let mut s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    s.view_mode = LibraryViewMode::List;
    s.rom_selected = 0;
    assert!(!s.rom_row_highlighted(0));
}

#[test]
fn rom_row_highlighted_only_in_games_mode() {
    let mut s = LibraryBrowseScreen::new(vec![], vec![], LEFT_PANEL_PERCENT_DEFAULT);
    s.view_mode = LibraryViewMode::Roms;
    s.rom_selected = 0;
    assert!(s.rom_row_highlighted(0));
    assert!(!s.rom_row_highlighted(1));

    s.rom_selected = 2;
    assert!(!s.rom_row_highlighted(0));
    assert!(s.rom_row_highlighted(2));
}
