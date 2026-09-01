use super::{
    background::types::{
        CollectionPrefetchDone, MetadataSearchDone, RomLoadDone, RomLoadEvent, SearchLoadDone,
        SearchLoadEvent,
    },
    event::{map_key_to_actions, Action, AppEvent, BackgroundAction},
    rom_load::{primary_rom_load_result_is_current, primary_rom_load_result_matches_selection},
    App, AppScreen,
};
use crate::tui::screens::connected_splash::StartupSplash;
use crate::tui::screens::game_detail::COVER_PANEL_WIDTH_DEFAULT;
use crate::tui::screens::library_browse::{LibraryBrowseScreen, LibrarySearchMode};
use crate::tui::screens::settings::{SettingsScreen, SettingsTab};
use crate::tui::screens::{GameDetailPrevious, GameDetailScreen, SearchScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use romm_api::client::RommClient;
use romm_api::config::LIBRARY_LEFT_PANEL_PERCENT_DEFAULT;
use romm_api::config::{default_theme_id, Config, ExtrasDefaults, TuiLayoutConfig};
use romm_api::core::cache::RomCacheKey;
use romm_api::feature_compat::{
    supported_achievements_compatibility, supported_metadata_edit_compatibility,
    supported_save_sync_compatibility,
};
use romm_api::types::{Platform, RomList, SearchRom};
use romm_api::update::UpdateStatus;
use serde_json::json;
use std::time::Instant;

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

fn app_with_library(platforms: Vec<Platform>) -> App {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let mut app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        None,
    );
    app.screen = AppScreen::LibraryBrowse(Box::new(LibraryBrowseScreen::new(
        platforms,
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    )));
    app
}

fn update_status_fixture() -> UpdateStatus {
    UpdateStatus {
        current_version: "0.25.0".into(),
        latest_version: "0.26.0".into(),
        release_tag: "v0.26.0".into(),
        should_update: true,
        release_url: "https://github.com/patricksmill/romm-cli/releases/tag/v0.26.0".into(),
        changelog_url: "https://github.com/patricksmill/romm-cli/blob/main/romm-tui/CHANGELOG.md"
            .into(),
    }
}

fn rom_fixture() -> romm_api::types::Rom {
    serde_json::from_value(json!({
        "id": 10,
        "platform_id": 1,
        "platform_slug": null,
        "platform_fs_slug": null,
        "platform_custom_name": null,
        "platform_display_name": null,
        "fs_name": "sample.zip",
        "fs_name_no_tags": "sample",
        "fs_name_no_ext": "sample",
        "fs_extension": "zip",
        "fs_path": "/sample.zip",
        "fs_size_bytes": 100,
        "name": "Sample",
        "slug": null,
        "summary": null,
        "path_cover_small": null,
        "path_cover_large": null,
        "url_cover": null,
        "has_manual": false,
        "path_manual": null,
        "url_manual": null,
        "is_unidentified": false,
        "is_identified": true
    }))
    .expect("valid rom fixture")
}

fn metadata_row(name: &str) -> SearchRom {
    serde_json::from_value(json!({
        "name": name,
        "platform_id": 1,
        "igdb_id": 5
    }))
    .expect("valid metadata row")
}

fn app_with_game_detail() -> App {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Search(SearchScreen::new()),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(detail));
    app
}

fn empty_rom_list_with_total(total: u64) -> RomList {
    RomList {
        items: vec![],
        total,
        limit: 50,
        offset: 0,
    }
}

#[tokio::test]
async fn list_move_to_zero_rom_selection_does_not_queue_deferred_load() {
    let mut app = app_with_library(vec![platform(1, "HasRoms", 5), platform(2, "Empty", 0)]);

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Down, KeyModifiers::empty()))
        .await
        .expect("key handled"));
    assert!(
        app.deferred_load_roms.is_none(),
        "selection move to zero-rom platform should not queue deferred ROM load"
    );
}

#[test]
fn ctrl_c_is_treated_as_force_quit() {
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(App::is_force_quit_key(&ctrl_c));

    let plain_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty());
    assert!(!App::is_force_quit_key(&plain_c));
}

#[test]
fn primary_rom_load_stale_gen_is_ignored() {
    assert!(!primary_rom_load_result_is_current(1, 2));
    assert!(primary_rom_load_result_is_current(3, 3));
}

#[test]
fn background_event_maps_to_background_action() {
    let app = app_with_library(vec![platform(1, "NES", 1)]);
    let actions = app.map_event(AppEvent::Background(BackgroundAction::PollFooterClear));
    assert!(matches!(
        actions.as_slice(),
        [Action::Background(BackgroundAction::PollFooterClear)]
    ));
}

#[test]
fn primary_rom_load_stale_key_does_not_match_selection() {
    let mut lib = LibraryBrowseScreen::new(
        vec![
            platform(1, "Nintendo 64", 312),
            platform(2, "Nintendo 3DS", 38),
        ],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    lib.list_index = 1;
    assert_eq!(
        lib.cache_key(),
        Some(RomCacheKey::Platform(2)),
        "fixture should select 3DS"
    );
    assert!(!primary_rom_load_result_matches_selection(
        &lib,
        &Some(RomCacheKey::Platform(1)),
    ));
    assert!(primary_rom_load_result_matches_selection(
        &lib,
        &Some(RomCacheKey::Platform(2)),
    ));
}

#[test]
fn primary_rom_load_batch_for_wrong_platform_is_ignored() {
    let mut app = app_with_library(vec![
        platform(1, "Nintendo 64", 312),
        platform(2, "Nintendo 3DS", 38),
    ]);
    if let AppScreen::LibraryBrowse(ref mut lib) = app.screen {
        lib.list_index = 1;
        lib.clear_roms();
        lib.set_rom_loading(true);
    }
    app.rom_load_gen = 1;
    app.rom_load_tx
        .send(RomLoadDone {
            gen: 1,
            key: Some(RomCacheKey::Platform(1)),
            expected: 312,
            event: RomLoadEvent::Batch(RomList {
                total: 1,
                limit: 1,
                offset: 0,
                items: vec![rom_fixture()],
            }),
            context: "test_stale_platform",
            started: Instant::now(),
        })
        .expect("send stale batch");

    app.poll_background_tasks();

    if let AppScreen::LibraryBrowse(ref lib) = app.screen {
        assert!(
            lib.roms.is_none(),
            "N64 batch must not populate games while 3DS is selected"
        );
    } else {
        panic!("expected library browse screen");
    }
}

#[test]
fn primary_rom_load_partial_batch_is_not_treated_as_cache_hit() {
    let mut app = app_with_library(vec![platform(1, "NES", 100)]);
    app.rom_load_gen = 1;
    app.rom_load_tx
        .send(RomLoadDone {
            gen: 1,
            key: Some(RomCacheKey::Platform(1)),
            expected: 100,
            event: RomLoadEvent::Batch(RomList {
                total: 100,
                limit: 50,
                offset: 0,
                items: vec![rom_fixture()],
            }),
            context: "test_partial_batch",
            started: Instant::now(),
        })
        .expect("send partial batch");

    app.poll_background_tasks();

    assert!(
        app.rom_cache
            .get_valid(&RomCacheKey::Platform(1), 100)
            .is_none(),
        "navigating away mid-pagination must not leave a truncated list as a valid cache hit"
    );
    assert_eq!(
        app.rom_partials
            .get(&RomCacheKey::Platform(1))
            .map(|(expected, list)| (*expected, list.items.len())),
        Some((100, 1)),
        "partial pages must remain available for resume"
    );
}

#[tokio::test]
async fn deferred_rom_load_seeds_ui_from_partial_and_resumes_offset() {
    let mut app = app_with_library(vec![platform(1, "NES", 100)]);
    let partial = RomList {
        total: 100,
        limit: 50,
        offset: 0,
        items: vec![rom_fixture(), rom_fixture()],
    };
    assert_eq!(partial.items.len(), 2);
    assert!(!romm_api::core::roms::rom_list_fetch_complete(&partial));
    app.rom_partials
        .insert(RomCacheKey::Platform(1), (100, partial));

    if let AppScreen::LibraryBrowse(ref mut lib) = app.screen {
        lib.clear_roms();
        lib.set_rom_loading(true);
    }

    let req = romm_api::endpoints::roms::GetRoms {
        platform_id: Some(1),
        limit: Some(50),
        ..Default::default()
    };
    app.deferred_load_roms = Some((
        Some(RomCacheKey::Platform(1)),
        Some(req),
        100,
        "test_resume_partial",
        Instant::now() - std::time::Duration::from_millis(300),
    ));
    app.process_deferred_rom_load_for_test();

    if let AppScreen::LibraryBrowse(ref lib) = app.screen {
        assert_eq!(
            lib.roms.as_ref().map(|r| r.items.len()),
            Some(2),
            "reselecting mid-fetch must restore partial progress, not restart empty"
        );
        assert!(lib.rom_loading, "resume should keep loading until complete");
    } else {
        panic!("expected library browse screen");
    }
    if let Some(task) = app.rom_load_task.take() {
        task.abort();
    }
}

#[test]
fn primary_rom_load_complete_batch_is_cached() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    app.rom_load_gen = 1;
    app.rom_load_tx
        .send(RomLoadDone {
            gen: 1,
            key: Some(RomCacheKey::Platform(1)),
            expected: 1,
            event: RomLoadEvent::Batch(RomList {
                total: 1,
                limit: 50,
                offset: 0,
                items: vec![rom_fixture()],
            }),
            context: "test_complete_batch",
            started: Instant::now(),
        })
        .expect("send complete batch");

    app.poll_background_tasks();

    assert!(
        app.rom_cache
            .get_valid(&RomCacheKey::Platform(1), 1)
            .is_some(),
        "finished ROM list should still be cached"
    );
}

#[test]
fn collection_prefetch_incomplete_list_is_not_disk_cached() {
    let mut app = app_with_library(vec![platform(1, "NES", 100)]);
    app.apply_background(BackgroundAction::CollectionPrefetch(
        CollectionPrefetchDone {
            key: RomCacheKey::Platform(1),
            expected: 100,
            roms: Some(RomList {
                total: 100,
                limit: 50,
                offset: 0,
                items: vec![rom_fixture()],
            }),
            warning: None,
        },
    ));
    assert!(app
        .rom_cache
        .get_valid(&RomCacheKey::Platform(1), 100)
        .is_none());
}

#[tokio::test]
async fn game_detail_esc_returns_to_previous_library_screen() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    let detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(detail));

    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("esc handled");
    assert!(!quit);
    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
}

#[tokio::test]
async fn game_detail_esc_resumes_partial_library_rom_load() {
    let mut app = app_with_library(vec![platform(1, "NES", 100)]);
    let mut partial = RomList {
        total: 100,
        limit: 50,
        offset: 0,
        items: vec![rom_fixture()],
    };
    partial.items[0].id = 42;

    let mut previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 100)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    previous.set_roms(partial.clone());
    previous.switch_view();
    app.rom_partials
        .insert(RomCacheKey::Platform(1), (100, partial));

    let detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(detail));

    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("esc handled");

    assert!(!quit);
    let Some((key, req, expected, context, _started)) = &app.deferred_load_roms else {
        panic!("restoring a partial loading library should queue a resumed ROM load");
    };
    assert_eq!(key, &Some(RomCacheKey::Platform(1)));
    assert_eq!(expected, &100);
    assert_eq!(context, &"restore_partial_library");
    assert!(req.as_ref().is_some_and(|r| r.platform_id == Some(1)));
}

#[tokio::test]
async fn startup_splash_enter_dismisses_without_quitting_when_update_pending() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let splash = Some(StartupSplash::new(
        config.base_url.clone(),
        Some("4.0.0".into()),
    ));
    let mut app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        Some("4.0.0".into()),
        splash,
        Some(update_status_fixture()),
    );
    assert!(app.startup_splash.is_some());
    assert!(app.startup_update_prompt.is_some());

    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .expect("enter handled");
    assert!(!quit, "Enter on connected splash should not quit the app");
    assert!(app.startup_splash.is_none(), "splash should be dismissed");
    assert!(
        app.startup_update_prompt.is_some(),
        "update prompt should remain after splash dismiss"
    );
}

#[tokio::test]
async fn startup_update_prompt_enter_starts_update_without_quitting() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let mut app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        Some(update_status_fixture()),
    );
    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .expect("enter handled");
    assert!(!quit, "Enter to confirm update should not quit the app");
    assert!(
        app.startup_update_prompt
            .as_ref()
            .is_some_and(|p| p.updating),
        "update should be in progress"
    );
}

#[tokio::test]
async fn startup_update_prompt_esc_skips_without_quitting() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let mut app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        Some(update_status_fixture()),
    );
    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("esc handled");
    assert!(!quit);
    assert!(app.startup_update_prompt.is_none());
}

#[test]
fn library_filter_bar_blocks_global_char_shortcuts() {
    let mut app = app_on_library();
    if let AppScreen::LibraryBrowse(ref mut lib) = app.screen {
        lib.enter_list_search(LibrarySearchMode::Filter);
    }
    assert!(app.blocks_global_d_shortcut());
    assert!(app.blocks_global_slash_shortcut());
    assert!(app.blocks_global_comma_shortcut());
    assert!(!app.allows_global_question_help());
}

#[test]
fn search_overlay_blocks_global_char_shortcuts() {
    let mut app = app_on_library();
    app.screen = AppScreen::Search(SearchScreen::new());
    assert!(app.blocks_global_d_shortcut());
    assert!(app.blocks_global_slash_shortcut());
    assert!(app.blocks_global_comma_shortcut());
    assert!(!app.allows_global_question_help());

    let d_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
    let actions = map_key_to_actions(&app, &d_key);
    assert!(
        matches!(actions.as_slice(), [Action::SearchKey(k)] if *k == d_key),
        "d should reach search input, not open downloads"
    );
}

#[tokio::test]
async fn startup_update_prompt_blocks_global_d_shortcut() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        Some(update_status_fixture()),
    );
    assert!(app.blocks_global_d_shortcut());
    assert!(app.blocks_global_chord_shortcuts());
}

#[tokio::test]
async fn startup_update_prompt_skip_closes_prompt() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let mut app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        Some(update_status_fixture()),
    );
    assert!(app.startup_update_prompt.is_some());
    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("esc handled");
    assert!(!quit);
    assert!(app.startup_update_prompt.is_none());
}

#[test]
fn search_batch_updates_results_without_stopping_loading() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let mut app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        None,
    );
    let mut search = SearchScreen::new();
    search.loading = true;
    app.screen = AppScreen::Search(search);

    app.search_load_tx
        .send(SearchLoadDone {
            query: "zelda".to_string(),
            event: SearchLoadEvent::Batch(empty_rom_list_with_total(120)),
        })
        .expect("send batch");

    app.poll_background_tasks();

    match &app.screen {
        AppScreen::Search(search) => {
            assert!(search.loading, "loading should continue after batch");
            assert!(search.results.is_some(), "batch should populate results");
            assert_eq!(search.last_searched_query.as_deref(), Some("zelda"));
        }
        _ => panic!("expected search screen"),
    }
}

#[test]
fn search_complete_event_stops_loading() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    let mut app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        None,
    );
    let mut search = SearchScreen::new();
    search.loading = true;
    app.screen = AppScreen::Search(search);

    app.search_load_tx
        .send(SearchLoadDone {
            query: "zelda".to_string(),
            event: SearchLoadEvent::Complete,
        })
        .expect("send complete");

    app.poll_background_tasks();

    match &app.screen {
        AppScreen::Search(search) => {
            assert!(!search.loading, "loading should stop after completion");
        }
        _ => panic!("expected search screen"),
    }
}

#[tokio::test]
async fn pressing_2_switches_to_extras_tab() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    let detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(detail));

    app.handle_key_event(&KeyEvent::new(KeyCode::Char('2'), KeyModifiers::empty()))
        .await
        .expect("handled");

    match &app.screen {
        AppScreen::GameDetail(d) => {
            assert_eq!(
                d.active_tab,
                crate::tui::screens::game_detail::DetailTab::Extras
            );
        }
        _ => panic!("expected game detail"),
    }
}

#[tokio::test]
async fn stale_metadata_search_result_from_previous_picker_is_ignored() {
    let mut app = app_with_game_detail();

    app.open_metadata_match_screen();
    let stale_gen = app.metadata_search_gen;

    app.handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("close first metadata picker");
    assert!(matches!(app.screen, AppScreen::GameDetail(_)));

    app.open_metadata_match_screen();

    app.apply_background(BackgroundAction::MetadataSearch(MetadataSearchDone {
        gen: stale_gen,
        rom_id: 10,
        result: Ok(vec![metadata_row("Wrong Game")]),
    }));

    match &app.screen {
        AppScreen::MetadataMatch(picker) => {
            assert!(
                matches!(
                    picker.phase,
                    crate::tui::screens::metadata_match::MetadataMatchPhase::QueryInput
                ),
                "stale results must not move the reopened picker out of query input"
            );
            assert!(picker.rows.is_empty(), "stale rows must not be shown");
        }
        _ => panic!("expected metadata picker"),
    }
}

#[tokio::test]
async fn metadata_match_cannot_start_while_apply_is_in_flight_for_same_rom() {
    let mut app = app_with_game_detail();
    app.metadata_apply_inflight_roms.insert(10);

    app.handle_key_event(&KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty()))
        .await
        .expect("handled metadata key");

    match &app.screen {
        AppScreen::GameDetail(detail) => {
            assert!(
                detail
                    .message
                    .as_deref()
                    .is_some_and(|msg| msg.contains("already in progress")),
                "user should be told the existing metadata update is still running"
            );
        }
        _ => panic!("metadata picker must not open while an update is in flight"),
    }
}

fn app_on_library() -> App {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).expect("client");
    App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        None,
    )
}

#[tokio::test]
async fn settings_theme_preview_reverts_when_leaving_without_save() {
    std::env::remove_var("NO_COLOR");
    let mut app = app_on_library();
    let saved_theme = app.config.theme.clone();
    assert_eq!(app.theme_id(), saved_theme);

    let mut settings = SettingsScreen::new(&app.config, None, supported_save_sync_compatibility());
    settings.selected_tab = SettingsTab::Appearance;
    app.screen = AppScreen::Settings(Box::new(settings));

    app.handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .expect("cycle theme");
    assert_ne!(app.theme_id(), saved_theme);
    assert_eq!(app.config.theme, saved_theme);

    app.handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("prompt to save");
    assert!(matches!(app.screen, AppScreen::Settings(_)));

    app.handle_key_event(&KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty()))
        .await
        .expect("discard and leave");

    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
    assert_eq!(app.theme_id(), saved_theme);
}

#[tokio::test]
async fn global_slash_toggles_search_overlay() {
    let mut app = app_on_library();
    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));

    app.handle_key_event(&KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()))
        .await
        .expect("open search");
    assert!(matches!(app.screen, AppScreen::Search(_)));

    app.handle_key_event(&KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()))
        .await
        .expect("type slash in query");
    if let AppScreen::Search(search) = &app.screen {
        assert_eq!(search.query, "/");
    } else {
        panic!("expected search overlay");
    }

    app.handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("close search");
    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
}

#[tokio::test]
async fn search_overlay_d_types_into_query_not_downloads() {
    let mut app = app_on_library();
    app.handle_key_event(&KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()))
        .await
        .expect("open search");

    app.handle_key_event(&KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()))
        .await
        .expect("type d in query");
    assert!(matches!(app.screen, AppScreen::Search(search) if search.query == "d"));
}

#[tokio::test]
async fn global_comma_toggles_settings_overlay() {
    let mut app = app_on_library();
    app.handle_key_event(&KeyEvent::new(KeyCode::Char(','), KeyModifiers::empty()))
        .await
        .expect("open settings");
    assert!(matches!(app.screen, AppScreen::Settings(_)));

    app.handle_key_event(&KeyEvent::new(KeyCode::Char(','), KeyModifiers::empty()))
        .await
        .expect("close settings");
    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
}

#[tokio::test]
async fn settings_exit_prompt_cancel_keeps_unsaved_preview() {
    std::env::remove_var("NO_COLOR");
    let mut app = app_on_library();
    let saved_theme = app.config.theme.clone();

    let mut settings = SettingsScreen::new(&app.config, None, supported_save_sync_compatibility());
    settings.selected_tab = SettingsTab::Appearance;
    app.screen = AppScreen::Settings(Box::new(settings));

    app.handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .expect("cycle theme");
    let preview_theme = app.theme_id().to_string();

    app.handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("prompt to save");
    app.handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("cancel prompt");

    assert!(matches!(app.screen, AppScreen::Settings(_)));
    assert_eq!(app.theme_id(), preview_theme);
    assert_eq!(app.config.theme, saved_theme);
}

#[tokio::test]
async fn settings_exit_without_changes_skips_prompt() {
    let mut app = app_on_library();
    app.screen = AppScreen::Settings(Box::new(SettingsScreen::new(
        &app.config,
        None,
        supported_save_sync_compatibility(),
    )));

    app.handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("leave settings");

    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
}

async fn apply_actions(app: &mut App, actions: Vec<Action>) -> bool {
    for action in actions {
        if app.update(action).await.expect("update") {
            return true;
        }
    }
    false
}

#[tokio::test]
async fn global_error_esc_dismisses_via_action_pipeline() {
    let mut app = app_on_library();
    app.global_error = Some("test error".into());
    let actions = map_key_to_actions(&app, &KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert!(matches!(actions.as_slice(), [Action::DismissGlobalMessage]));
    apply_actions(&mut app, actions).await;
    assert!(app.global_error.is_none());
}

#[tokio::test]
async fn library_quit_maps_to_quit_action() {
    let app = app_on_library();
    let actions = map_key_to_actions(
        &app,
        &KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
    );
    assert!(matches!(actions.as_slice(), [Action::LibraryKey(_)]));
    let mut app = app;
    assert!(app
        .handle_key_event(&KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()))
        .await
        .expect("quit"));
}
