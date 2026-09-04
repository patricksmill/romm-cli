use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use romm_api::client::RommClient;
use romm_api::config::LIBRARY_LEFT_PANEL_PERCENT_DEFAULT;
use romm_api::config::{default_theme_id, Config, ExtrasDefaults, TuiLayoutConfig};
use romm_api::core::utils;
use romm_api::feature_compat::{
    supported_achievements_compatibility, supported_metadata_edit_compatibility,
    supported_save_sync_compatibility,
};
use romm_api::types::{Rom, RomList};
use romm_tui::tui::app::{App, AppScreen};
use romm_tui::tui::screens::library_browse::{LibraryBrowseScreen, LibraryViewMode};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn startup_library_api_error_shows_footer() {
    let mock_server = MockServer::start().await;
    for api_path in [
        "/api/collections",
        "/api/collections/smart",
        "/api/collections/virtual",
    ] {
        Mock::given(method("GET"))
            .and(path(api_path))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/api/platforms"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let config = Config {
        base_url: mock_server.uri(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).unwrap();
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
    app.open_library_browse();

    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
    assert!(app.global_error.is_none());

    let mut saw_failure_footer = false;
    for _ in 0..80 {
        app.poll_background_tasks();
        if let AppScreen::LibraryBrowse(ref lib) = app.screen {
            if let Some(ref foot) = lib.metadata_footer {
                if foot.contains("500")
                    || foot.contains("Partial refresh")
                    || foot.contains("Could not refresh library metadata")
                {
                    saw_failure_footer = true;
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        saw_failure_footer,
        "expected metadata footer to mention API failure after background refresh"
    );
}

#[tokio::test]
async fn startup_opens_library_browse() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/platforms"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/collections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/collections/smart"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/collections/virtual"))
        .and(query_param("type", "all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let config = Config {
        base_url: mock_server.uri(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).unwrap();
    let app = App::new(
        client,
        config,
        supported_save_sync_compatibility(),
        supported_metadata_edit_compatibility(),
        supported_achievements_compatibility(),
        None,
        None,
        None,
    );

    assert!(app.global_error.is_none());
    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
}

#[tokio::test]
async fn library_esc_quits_from_list_view() {
    let mock_server = MockServer::start().await;
    let config = Config {
        base_url: mock_server.uri(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).unwrap();
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

    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .unwrap();
    assert!(quit, "expected Esc at library root to quit");
}

#[tokio::test]
async fn global_d_opens_download_overlay_from_library() {
    let mock_server = MockServer::start().await;
    let config = Config {
        base_url: mock_server.uri(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).unwrap();
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

    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()))
        .await
        .unwrap();
    assert!(!quit);
    assert!(
        matches!(app.screen, AppScreen::Download(_)),
        "expected 'd' to open Download overlay"
    );
}

#[tokio::test]
async fn global_slash_opens_search_overlay() {
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
    let client = RommClient::new(&config, false).unwrap();
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

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()))
        .await
        .unwrap());
    assert!(matches!(app.screen, AppScreen::Search(_)));
}

fn sample_rom(id: u64, name: &str) -> Rom {
    Rom {
        id,
        platform_id: 1,
        platform_slug: None,
        platform_fs_slug: None,
        platform_custom_name: None,
        platform_display_name: None,
        fs_name: format!("{name}.zip"),
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

#[tokio::test]
async fn library_enter_opens_game_detail() {
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
    let client = RommClient::new(&config, false).unwrap();
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

    let items = vec![sample_rom(1, "alpha"), sample_rom(2, "beta")];
    let rom_list = RomList {
        total: items.len() as u64,
        limit: items.len() as u64,
        offset: 0,
        items: items.clone(),
    };

    let mut lib = LibraryBrowseScreen::new(vec![], vec![], LIBRARY_LEFT_PANEL_PERCENT_DEFAULT);
    lib.roms = Some(rom_list);
    lib.rom_groups = Some(utils::group_roms_by_name(&items));
    lib.view_mode = LibraryViewMode::Roms;
    app.screen = AppScreen::LibraryBrowse(Box::new(lib));

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap());
    assert!(
        matches!(&app.screen, AppScreen::GameDetail(d) if d.rom.name == "alpha"),
        "Enter should open the selected game"
    );
}

#[tokio::test]
async fn game_detail_download_is_blocked_when_config_download_path_is_invalid() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("romm-invalid-dl-{ts}"));
    std::fs::create_dir_all(&base).unwrap();
    let invalid_target = base.join("not-a-directory.txt");
    std::fs::write(&invalid_target, b"x").unwrap();

    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: invalid_target.to_string_lossy().to_string(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).unwrap();
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

    let items = vec![sample_rom(1, "alpha")];
    let rom_list = RomList {
        total: items.len() as u64,
        limit: items.len() as u64,
        offset: 0,
        items: items.clone(),
    };

    let mut lib = LibraryBrowseScreen::new(vec![], vec![], LIBRARY_LEFT_PANEL_PERCENT_DEFAULT);
    lib.roms = Some(rom_list);
    lib.rom_groups = Some(utils::group_roms_by_name(&items));
    lib.view_mode = LibraryViewMode::Roms;
    app.screen = AppScreen::LibraryBrowse(Box::new(lib));

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap());
    assert!(matches!(&app.screen, AppScreen::GameDetail(_)));

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap());
    assert!(
        matches!(&app.screen, AppScreen::GameDetail(d) if d.message.as_deref().is_some_and(|m| m.contains("Download blocked"))),
        "invalid configured download path should block start with a user-facing message"
    );

    let _ = std::fs::remove_file(&invalid_target);
    let _ = std::fs::remove_dir_all(&base);
}

#[tokio::test]
async fn game_detail_download_skips_when_rom_already_exists_in_console_folder() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let roms_dir = std::env::temp_dir().join(format!("romm-roms-dir-{ts}"));
    let console_dir = roms_dir.join("platform-1");
    std::fs::create_dir_all(&console_dir).unwrap();
    std::fs::write(console_dir.join("alpha.zip"), b"x").unwrap();

    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: roms_dir.to_string_lossy().to_string(),
        use_https: false,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: Default::default(),
        roms_layout: Default::default(),
        theme: default_theme_id(),
        tui_layout: TuiLayoutConfig::default(),
    };
    let client = RommClient::new(&config, false).unwrap();
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

    let items = vec![sample_rom(1, "alpha")];
    let rom_list = RomList {
        total: items.len() as u64,
        limit: items.len() as u64,
        offset: 0,
        items: items.clone(),
    };

    let mut lib = LibraryBrowseScreen::new(vec![], vec![], LIBRARY_LEFT_PANEL_PERCENT_DEFAULT);
    lib.roms = Some(rom_list);
    lib.rom_groups = Some(utils::group_roms_by_name(&items));
    lib.view_mode = LibraryViewMode::Roms;
    app.screen = AppScreen::LibraryBrowse(Box::new(lib));

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap());
    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap());

    let mut saw_skip = false;
    for _ in 0..50 {
        if let AppScreen::GameDetail(detail) = &app.screen {
            if let Ok(list) = detail.downloads.lock() {
                saw_skip = list.iter().any(|j| {
                    j.rom_id == 1
                        && matches!(
                            j.status,
                            romm_api::core::download::DownloadStatus::SkippedAlreadyExists
                        )
                });
            }
        }
        if saw_skip {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    assert!(
        saw_skip,
        "expected existing ROM to produce SkippedAlreadyExists status"
    );
    let _ = std::fs::remove_dir_all(&roms_dir);
}
