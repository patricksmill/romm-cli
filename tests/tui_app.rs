use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use romm_cli::client::RommClient;
use romm_cli::config::Config;
use romm_cli::core::utils;
use romm_cli::tui::app::{App, AppScreen};
use romm_cli::tui::openapi::EndpointRegistry;
use romm_cli::tui::screens::library_browse::{
    LibraryBrowseScreen, LibrarySearchMode, LibraryViewMode,
};
use romm_cli::types::{Rom, RomList};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_main_menu_api_error_shows_popup() {
    let mock_server = MockServer::start().await;
    // Background library refresh uses collection summary endpoints (not GET /api/platforms).
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
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None, None);

    // Simulate pressing Enter on Main Menu (Platforms)
    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap();
    assert!(!quit);

    // Library opens immediately; metadata refresh runs in background.
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
async fn test_main_menu_success_transitions_to_library() {
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
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None, None);

    // Simulate pressing Enter on Main Menu (Platforms)
    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap();
    assert!(!quit);

    // Assert error is not set
    assert!(app.global_error.is_none());

    // Assert we transitioned to LibraryBrowse
    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
}

#[tokio::test]
async fn main_menu_fifth_item_is_exit() {
    let mock_server = MockServer::start().await;
    let config = Config {
        base_url: mock_server.uri(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None, None);

    // Move to the 5th menu row (0-based index 4).
    for _ in 0..4 {
        assert!(!app
            .handle_key_event(&KeyEvent::new(KeyCode::Down, KeyModifiers::empty()))
            .await
            .unwrap());
    }

    // Without API (Expert) in the menu, the 5th item should be Exit.
    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap();
    assert!(quit, "expected Enter on 5th item to quit");
}

#[tokio::test]
async fn library_filter_mode_d_types_in_search_bar_not_downloads() {
    let mock_server = MockServer::start().await;
    let config = Config {
        base_url: mock_server.uri(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None, None);

    let mut lib = LibraryBrowseScreen::new(vec![], vec![]);
    lib.view_mode = LibraryViewMode::Roms;
    lib.enter_rom_search(LibrarySearchMode::Filter);
    app.screen = AppScreen::LibraryBrowse(lib);

    let quit = app
        .handle_key_event(&KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()))
        .await
        .unwrap();
    assert!(!quit);
    assert!(
        matches!(&app.screen, AppScreen::LibraryBrowse(l) if l.rom_search.query == "d" && l.rom_search.mode.is_some()),
        "expected 'd' in filter bar, not Download overlay"
    );
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
        is_unidentified: false,
        is_identified: true,
    }
}

#[tokio::test]
async fn library_filter_enter_then_enter_opens_game_detail() {
    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None, None);

    let items = vec![sample_rom(1, "alpha"), sample_rom(2, "beta")];
    let rom_list = RomList {
        total: items.len() as u64,
        limit: items.len() as u64,
        offset: 0,
        items: items.clone(),
    };

    let mut lib = LibraryBrowseScreen::new(vec![], vec![]);
    lib.roms = Some(rom_list);
    lib.rom_groups = Some(utils::group_roms_by_name(&items));
    lib.view_mode = LibraryViewMode::Roms;
    lib.enter_rom_search(LibrarySearchMode::Filter);
    for c in "al".chars() {
        lib.add_rom_search_char(c);
    }
    app.screen = AppScreen::LibraryBrowse(lib);

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap());
    assert!(
        matches!(&app.screen, AppScreen::LibraryBrowse(l) if l.rom_search.filter_browsing && l.rom_search.mode.is_none()),
        "first Enter should commit filter browsing"
    );

    assert!(!app
        .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .unwrap());
    assert!(
        matches!(&app.screen, AppScreen::GameDetail(d) if d.rom.name == "alpha"),
        "second Enter should open the selected filtered game"
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
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None, None);

    let items = vec![sample_rom(1, "alpha")];
    let rom_list = RomList {
        total: items.len() as u64,
        limit: items.len() as u64,
        offset: 0,
        items: items.clone(),
    };

    let mut lib = LibraryBrowseScreen::new(vec![], vec![]);
    lib.roms = Some(rom_list);
    lib.rom_groups = Some(utils::group_roms_by_name(&items));
    lib.view_mode = LibraryViewMode::Roms;
    app.screen = AppScreen::LibraryBrowse(lib);

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
    std::fs::write(console_dir.join("alpha.zip"), b"existing").unwrap();

    let config = Config {
        base_url: "http://127.0.0.1:9".into(),
        download_dir: roms_dir.to_string_lossy().to_string(),
        use_https: false,
        auth: None,
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None, None);

    let items = vec![sample_rom(1, "alpha")];
    let rom_list = RomList {
        total: items.len() as u64,
        limit: items.len() as u64,
        offset: 0,
        items: items.clone(),
    };

    let mut lib = LibraryBrowseScreen::new(vec![], vec![]);
    lib.roms = Some(rom_list);
    lib.rom_groups = Some(utils::group_roms_by_name(&items));
    lib.view_mode = LibraryViewMode::Roms;
    app.screen = AppScreen::LibraryBrowse(lib);

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
                            romm_cli::core::download::DownloadStatus::SkippedAlreadyExists
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
