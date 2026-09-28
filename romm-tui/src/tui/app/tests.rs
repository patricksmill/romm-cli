use super::{
    background::types::{
        AchievementLoadDone, CollectionPrefetchDone, LibraryMetadataRefreshDone, MetadataApplyDone,
        MetadataSearchDone, RomLoadDone, RomLoadEvent, SaveListDone, SearchLoadDone,
        SearchLoadEvent,
    },
    event::{map_key_to_actions, Action, AppEvent, BackgroundAction},
    rom_load::{primary_rom_load_result_is_current, primary_rom_load_result_matches_selection},
    App, AppScreen,
};
use crate::tui::screens::connected_splash::StartupSplash;
use crate::tui::screens::game_detail::{
    AchievementListState, SaveListState, COVER_PANEL_WIDTH_DEFAULT,
};
use crate::tui::screens::library_browse::{LibraryBrowseScreen, LibrarySearchMode};
use crate::tui::screens::settings::{SettingsConfirm, SettingsScreen, SettingsTab};
use crate::tui::screens::{
    GameDetailPrevious, GameDetailScreen, MetadataMatchScreen, SearchScreen,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use romm_api::client::RommClient;
use romm_api::config::{
    default_theme_id, AuthConfig, Config, ExtrasDefaults, TuiLayoutConfig,
    KEYRING_SECRET_PLACEHOLDER, LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
};
use romm_api::core::cache::RomCacheKey;
use romm_api::core::{library_scan::ScanCacheInvalidate, startup_library_snapshot};
use romm_api::feature_compat::{
    supported_achievements_compatibility, supported_metadata_edit_compatibility,
    supported_save_sync_compatibility,
};
use romm_api::types::{Platform, RomList, SearchRom};
use romm_api::update::UpdateStatus;
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

fn app_with_library_base_url(platforms: Vec<Platform>, base_url: String) -> App {
    let config = Config {
        base_url,
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

fn app_with_library(platforms: Vec<Platform>) -> App {
    app_with_library_base_url(platforms, "http://127.0.0.1:9".into())
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

fn rom_fixture_json(id: u64) -> serde_json::Value {
    json!({
        "id": id,
        "platform_id": 1,
        "platform_slug": null,
        "platform_fs_slug": null,
        "platform_custom_name": null,
        "platform_display_name": null,
        "fs_name": format!("sample-{id}.zip"),
        "fs_name_no_tags": format!("sample-{id}"),
        "fs_name_no_ext": format!("sample-{id}"),
        "fs_extension": "zip",
        "fs_path": format!("/sample-{id}.zip"),
        "fs_size_bytes": 100,
        "name": format!("Sample {id}"),
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
    })
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

async fn spawn_search_server() -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind search server");
    let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = requests.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            let recorded = recorded.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let Ok(n) = stream.read(&mut buf).await else {
                    return;
                };
                let req = String::from_utf8_lossy(&buf[..n]);
                let first_line = req.lines().next().unwrap_or_default().to_string();
                recorded
                    .lock()
                    .expect("record request")
                    .push(first_line.clone());

                let offset = if first_line.contains("offset=20000") {
                    20000
                } else {
                    0
                };
                let item_count = if offset == 0 { 20_000 } else { 50 };
                let items = (0..item_count)
                    .map(|i| rom_fixture_json(offset + i))
                    .collect::<Vec<_>>();
                let body = json!({
                    "items": items,
                    "total": 20_050,
                    "limit": 50,
                    "offset": offset
                })
                .to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    (base_url, requests)
}

#[test]
fn successful_empty_metadata_refresh_clears_stale_platforms() {
    let snapshot_path = std::env::temp_dir().join(format!(
        "romm-empty-refresh-snapshot-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::env::set_var("ROMM_LIBRARY_METADATA_SNAPSHOT_PATH", &snapshot_path);

    let mut app = app_with_library(vec![platform(1, "Stale", 1)]);
    if let AppScreen::LibraryBrowse(ref mut lib) = app.screen {
        lib.set_roms(empty_rom_list_with_total(1));
    }
    app.library_metadata_refresh_gen = 7;

    app.apply_background(BackgroundAction::LibraryMetadataRefresh(
        LibraryMetadataRefreshDone {
            gen: 7,
            platforms: Vec::new(),
            collections: Vec::new(),
            collection_digest: Vec::new(),
            warnings: Vec::new(),
        },
    ));

    std::env::remove_var("ROMM_LIBRARY_METADATA_SNAPSHOT_PATH");
    let _ = std::fs::remove_file(snapshot_path);

    match &app.screen {
        AppScreen::LibraryBrowse(lib) => {
            assert!(lib.platforms.is_empty());
            assert!(lib.collections.is_empty());
            assert!(lib.roms.is_none());
            assert!(lib.cache_key().is_none());
        }
        _ => panic!("expected library browse"),
    }
}

struct EnvOverride {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvOverride {
    fn set_path(key: &'static str, value: &std::path::Path) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvOverride {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

struct TestConfigDir {
    _guard: std::sync::MutexGuard<'static, ()>,
    dir: PathBuf,
    previous: Option<String>,
}

impl TestConfigDir {
    fn new() -> Self {
        let guard = romm_api::config::test_env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let previous = std::env::var("ROMM_TEST_CONFIG_DIR").ok();
        let dir = std::env::temp_dir().join(format!(
            "romm-tui-app-config-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create test config dir");
        std::env::set_var("ROMM_TEST_CONFIG_DIR", &dir);
        Self {
            _guard: guard,
            dir,
            previous,
        }
    }

    fn config_path(&self) -> PathBuf {
        self.dir.join("config.json")
    }
}

impl Drop for TestConfigDir {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var("ROMM_TEST_CONFIG_DIR", value),
            None => std::env::remove_var("ROMM_TEST_CONFIG_DIR"),
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn layout_persistence_preserves_disk_keyring_auth_when_effective_auth_missing() {
    let env = TestConfigDir::new();
    std::fs::write(
        env.config_path(),
        r#"{
            "base_url": "http://example.test",
            "download_dir": "/tmp",
            "use_https": false,
            "auth": { "ApiKey": { "header": "X-Api-Key", "key": "<stored-in-keyring>" } }
        }"#,
    )
    .expect("write disk config");

    let mut app = app_on_library();
    app.config.auth = None;
    app.config.tui_layout.library_left_panel_percent = 35;

    app.persist_tui_layout();

    let saved: Config =
        serde_json::from_str(&std::fs::read_to_string(env.config_path()).expect("read config"))
            .expect("parse config");
    match saved.auth {
        Some(AuthConfig::ApiKey { header, key }) => {
            assert_eq!(header, "X-Api-Key");
            assert_eq!(key, KEYRING_SECRET_PLACEHOLDER);
        }
        other => panic!("expected API key sentinel auth preserved, got {other:?}"),
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

#[test]
fn primary_rom_load_batch_updates_partial_while_in_game_detail() {
    let mut app = app_with_library(vec![platform(1, "NES", 100)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 100)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    )));
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
                items: vec![rom_fixture(), rom_fixture()],
            }),
            context: "test_off_library_partial",
            started: Instant::now(),
        })
        .expect("send off-library batch");

    app.poll_background_tasks();

    assert_eq!(
        app.rom_partials
            .get(&RomCacheKey::Platform(1))
            .map(|(expected, list)| (*expected, list.items.len())),
        Some((100, 2)),
        "off-library batches should still advance resumable partial progress"
    );
}

#[test]
fn primary_rom_load_complete_batch_is_cached_while_in_game_detail() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    )));
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
            context: "test_off_library_complete",
            started: Instant::now(),
        })
        .expect("send off-library complete batch");

    app.poll_background_tasks();

    assert!(
        app.rom_cache
            .get_valid(&RomCacheKey::Platform(1), 1)
            .is_some(),
        "off-library complete batches should still warm the ROM cache"
    );
}

#[test]
fn settings_clear_cache_drops_in_memory_rom_partials() {
    let _env_lock = romm_api::config::test_env_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let cache_path = std::env::temp_dir().join(format!(
        "romm-tui-cache-clear-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_nanos()
    ));
    let _cache_env = EnvOverride::set_path("ROMM_CACHE_PATH", &cache_path);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let mut app = app_with_library(vec![platform(1, "NES", 100)]);
        app.rom_partials.insert(
            RomCacheKey::Platform(1),
            (100, empty_rom_list_with_total(100)),
        );

        let mut settings =
            SettingsScreen::new(&app.config, None, supported_save_sync_compatibility());
        settings.confirm = Some(SettingsConfirm::ClearCache);
        app.screen = AppScreen::Settings(Box::new(settings));

        let quit = app
            .handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
            .await
            .expect("clear cache");

        assert!(!quit);
        assert!(
            app.rom_partials.is_empty(),
            "Clear cache must also drop in-memory partial ROM lists"
        );
    });

    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn forced_metadata_rom_reload_drops_matching_partial_before_requeue() {
    let mut app = app_with_library(vec![platform(1, "NES", 100)]);
    app.rom_partials.insert(
        RomCacheKey::Platform(1),
        (100, empty_rom_list_with_total(100)),
    );
    app.force_rom_reload_after_metadata = true;

    app.apply_background(BackgroundAction::LibraryMetadataRefresh(
        LibraryMetadataRefreshDone {
            gen: app.library_metadata_refresh_gen,
            platforms: vec![platform(1, "NES", 100)],
            collections: Vec::new(),
            collection_digest: Vec::new(),
            warnings: Vec::new(),
        },
    ));

    assert!(
        !app.rom_partials.contains_key(&RomCacheKey::Platform(1)),
        "forced metadata reload must start from a fresh ROM list"
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
fn primary_rom_load_ceiling_batch_is_not_disk_cached() {
    let total = romm_api::core::roms::ROM_PAGE_CEILING + 1;
    let mut app = app_with_library(vec![platform(1, "NES", total)]);
    app.rom_load_gen = 1;
    app.rom_load_tx
        .send(RomLoadDone {
            gen: 1,
            key: Some(RomCacheKey::Platform(1)),
            expected: total,
            event: RomLoadEvent::Batch(RomList {
                total,
                limit: 50,
                offset: 0,
                items: (0..romm_api::core::roms::ROM_PAGE_CEILING)
                    .map(|_| rom_fixture())
                    .collect(),
            }),
            context: "test_ceiling_batch",
            started: Instant::now(),
        })
        .expect("send ceiling batch");

    app.poll_background_tasks();

    assert!(
        app.rom_cache
            .get_valid(&RomCacheKey::Platform(1), total)
            .is_none(),
        "safety-capped lists must not persist as complete cache entries"
    );
}

#[tokio::test]
async fn deferred_rom_load_empty_page_before_total_fails_instead_of_completing() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/roms"))
        .and(query_param("platform_ids", "1"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [],
            "total": 100,
            "limit": 50,
            "offset": 0
        })))
        .mount(&mock_server)
        .await;

    let mut app = app_with_library_base_url(vec![platform(1, "NES", 100)], mock_server.uri());
    let req = romm_api::endpoints::roms::GetRoms {
        platform_id: Some(1),
        limit: Some(50),
        ..Default::default()
    };
    app.deferred_load_roms = Some((
        Some(RomCacheKey::Platform(1)),
        Some(req),
        100,
        "test_empty_page",
        Instant::now() - Duration::from_millis(300),
    ));

    app.process_deferred_rom_load_for_test();

    let mut saw_failure_footer = false;
    let mut last_state = String::new();
    for _ in 0..40 {
        app.poll_background_tasks();
        if let AppScreen::LibraryBrowse(ref lib) = app.screen {
            last_state = format!(
                "loading={}, footer={:?}, rom_count={:?}, cache_key={:?}",
                lib.rom_loading,
                lib.metadata_footer,
                lib.roms.as_ref().map(|roms| roms.items.len()),
                lib.cache_key()
            );
            if !lib.rom_loading
                && lib
                    .metadata_footer
                    .as_deref()
                    .is_some_and(|footer| footer.contains("Could not load games"))
            {
                saw_failure_footer = true;
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    assert!(
        saw_failure_footer,
        "empty page before total should surface a load failure instead of silently completing; last state: {last_state}"
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
async fn metadata_apply_success_marks_previous_library_stale_and_clears_live_cache() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let stale_list = RomList {
        total: 1,
        limit: 50,
        offset: 0,
        items: vec![rom_fixture()],
    };
    let partial_list = RomList {
        total: 2,
        limit: 50,
        offset: 0,
        items: vec![rom_fixture()],
    };

    let mut previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    previous.set_roms(stale_list.clone());
    app.rom_cache
        .insert(RomCacheKey::Platform(1), stale_list.clone(), 1);
    app.rom_cache
        .insert(RomCacheKey::Collection(9), stale_list, 1);
    app.rom_partials
        .insert(RomCacheKey::Platform(1), (2, partial_list));

    let detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(detail));

    let mut updated = rom_fixture();
    updated.name = "Updated Metadata".into();
    app.apply_background(BackgroundAction::MetadataApply(MetadataApplyDone {
        rom_id: updated.id,
        platform_id: updated.platform_id,
        result: Ok(Box::new(updated)),
    }));

    assert!(app
        .rom_cache
        .get_valid(&RomCacheKey::Platform(1), 1)
        .is_none());
    assert!(app
        .rom_cache
        .get_valid(&RomCacheKey::Collection(9), 1)
        .is_none());
    assert!(!app.rom_partials.contains_key(&RomCacheKey::Platform(1)));

    let AppScreen::GameDetail(detail) = &app.screen else {
        panic!("expected game detail after metadata apply");
    };
    assert_eq!(detail.rom.name, "Updated Metadata");
    let GameDetailPrevious::Library(previous) = &detail.previous else {
        panic!("expected previous library screen");
    };
    assert!(
        previous.roms.is_none(),
        "stale library ROM list should be cleared after metadata apply"
    );
    assert!(
        previous.rom_loading,
        "returning to the previous library should trigger a fresh load"
    );
}

#[tokio::test]
async fn scan_completed_off_library_reloads_complete_roms_when_returning_to_library() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let mut previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    previous.set_roms(RomList {
        total: 1,
        limit: 50,
        offset: 0,
        items: vec![rom_fixture()],
    });

    let detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    app.screen = AppScreen::GameDetail(Box::new(detail));
    app.library_scan_pending_invalidate = Some(ScanCacheInvalidate::Platform(1));

    app.on_library_scan_completed_success();
    app.handle_key_event(&KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()))
        .await
        .expect("esc handled");

    assert!(
        app.force_rom_reload_after_metadata,
        "scan completion off Library must mark the restored complete ROM list stale"
    );
    assert!(
        app.library_metadata_refresh_gen > 0,
        "returning to Library after an off-screen scan must refresh metadata"
    );

    let gen = app.library_metadata_refresh_gen;
    app.apply_library_metadata_refresh(LibraryMetadataRefreshDone {
        gen,
        platforms: vec![platform(1, "NES", 1)],
        collections: vec![],
        collection_digest: startup_library_snapshot::build_collection_digest_from_collections(&[]),
        warnings: vec![],
    });

    match &app.screen {
        AppScreen::LibraryBrowse(lib) => {
            assert!(
                lib.roms.is_none(),
                "metadata refresh after scan must clear the stale complete ROM list"
            );
            assert!(lib.rom_loading, "cleared ROM pane should show reload state");
        }
        _ => panic!("expected restored library screen"),
    }
    let Some((key, req, expected, context, _started)) = &app.deferred_load_roms else {
        panic!("post-scan metadata refresh should queue a fresh ROM load");
    };
    assert_eq!(key, &Some(RomCacheKey::Platform(1)));
    assert_eq!(expected, &1);
    assert_eq!(context, &"post_scan_reload");
    assert!(req.as_ref().is_some_and(|r| r.platform_id == Some(1)));
}

#[tokio::test]
async fn metadata_apply_refreshes_achievement_state() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    let mut detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    detail.apply_achievements_empty("Not matched to RetroAchievements".into());
    app.screen = AppScreen::GameDetail(Box::new(detail));

    let mut refreshed = rom_fixture();
    refreshed.ra_id = Some(1234);
    app.apply_background(BackgroundAction::MetadataApply(MetadataApplyDone {
        rom_id: refreshed.id,
        platform_id: refreshed.platform_id,
        result: Ok(Box::new(refreshed)),
    }));

    match &app.screen {
        AppScreen::GameDetail(detail) => {
            assert_eq!(
                format!("{:?}", detail.achievements_state),
                "Loading",
                "metadata updates can change RA linkage, so achievements must be refreshed"
            );
        }
        _ => panic!("expected game detail"),
    }
}

#[tokio::test]
async fn stale_metadata_apply_does_not_refresh_current_achievements() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    let mut current_rom = rom_fixture();
    current_rom.id = 20;
    let mut detail = GameDetailScreen::new(
        current_rom,
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    detail.apply_achievements_empty("Current achievement state".into());
    app.screen = AppScreen::GameDetail(Box::new(detail));

    let mut stale_rom = rom_fixture();
    stale_rom.id = 10;
    app.apply_background(BackgroundAction::MetadataApply(MetadataApplyDone {
        rom_id: stale_rom.id,
        platform_id: stale_rom.platform_id,
        result: Ok(Box::new(stale_rom)),
    }));

    match &app.screen {
        AppScreen::GameDetail(detail) => {
            assert_eq!(
                format!("{:?}", detail.achievements_state),
                "Empty(\"Current achievement state\")",
                "a metadata completion for another ROM must not reload the visible game's achievements"
            );
        }
        _ => panic!("expected game detail"),
    }
}

#[tokio::test]
async fn stale_metadata_apply_does_not_close_current_metadata_picker() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    let mut current_rom = rom_fixture();
    current_rom.id = 20;
    let detail = GameDetailScreen::new(
        current_rom,
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    app.screen =
        AppScreen::MetadataMatch(Box::new(MetadataMatchScreen::new_for_rom(Box::new(detail))));

    let mut stale_rom = rom_fixture();
    stale_rom.id = 10;
    app.apply_background(BackgroundAction::MetadataApply(MetadataApplyDone {
        rom_id: stale_rom.id,
        platform_id: stale_rom.platform_id,
        result: Ok(Box::new(stale_rom)),
    }));

    match &app.screen {
        AppScreen::MetadataMatch(picker) => {
            assert_eq!(
                picker.previous.rom.id, 20,
                "a stale metadata completion must not close or replace the active picker"
            );
        }
        _ => panic!("expected metadata match screen"),
    }
}

#[test]
fn stale_achievement_load_result_is_ignored() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    let mut detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    detail.set_achievements_loading();
    app.screen = AppScreen::GameDetail(Box::new(detail));
    app.achievement_load_gen = 2;

    app.apply_background(BackgroundAction::AchievementLoad(AchievementLoadDone {
        rom_id: 10,
        gen: 1,
        result: Ok(romm_api::core::achievements::AchievementLoadResult::Empty(
            "Old result".into(),
        )),
    }));

    match &app.screen {
        AppScreen::GameDetail(detail) => {
            assert_eq!(
                format!("{:?}", detail.achievements_state),
                "Loading",
                "an older achievement worker must not overwrite a newer refresh"
            );
        }
        _ => panic!("expected game detail"),
    }
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
    app.search_load_gen = 1;

    app.search_load_tx
        .send(SearchLoadDone {
            gen: 1,
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
    app.search_load_gen = 1;

    app.search_load_tx
        .send(SearchLoadDone {
            gen: 1,
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

#[test]
fn stale_search_load_events_are_ignored() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let mut search = SearchScreen::new();
    search.query = "zelda".to_string();
    search.cursor_pos = search.query.len();
    search.loading = true;
    search.set_results_for_query("zelda".to_string(), empty_rom_list_with_total(0));
    app.screen = AppScreen::Search(search);
    app.search_load_gen = 2;

    app.search_load_tx
        .send(SearchLoadDone {
            gen: 1,
            query: "mario".to_string(),
            event: SearchLoadEvent::Batch(empty_rom_list_with_total(120)),
        })
        .expect("send stale batch");
    app.search_load_tx
        .send(SearchLoadDone {
            gen: 1,
            query: "mario".to_string(),
            event: SearchLoadEvent::Complete,
        })
        .expect("send stale complete");

    app.poll_background_tasks();

    match &app.screen {
        AppScreen::Search(search) => {
            assert!(
                search.loading,
                "stale completion must not stop current search"
            );
            assert_eq!(search.last_searched_query.as_deref(), Some("zelda"));
            assert_eq!(search.results.as_ref().map(|r| r.total), Some(0));
        }
        _ => panic!("expected search screen"),
    }
}

#[tokio::test]
async fn global_search_stops_fetching_at_rom_page_ceiling() {
    let (base_url, requests) = spawn_search_server().await;
    let config = Config {
        base_url,
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
    search.query = "sample".into();
    search.cursor_pos = search.query.len();
    app.screen = AppScreen::Search(search);

    app.handle_search(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .expect("start search");

    for _ in 0..100 {
        app.poll_background_tasks();
        if matches!(&app.screen, AppScreen::Search(search) if !search.loading) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    match &app.screen {
        AppScreen::Search(search) => {
            assert!(!search.loading, "search should complete after capped batch");
            assert_eq!(
                search.results.as_ref().map(|r| r.items.len()),
                Some(romm_api::core::roms::ROM_PAGE_CEILING as usize)
            );
        }
        _ => panic!("expected search screen"),
    }

    let requests = requests.lock().expect("requests").clone();
    assert!(
        !requests.iter().any(|line| line.contains("offset=20000")),
        "search worker fetched beyond the page ceiling: {requests:?}"
    );
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

#[test]
fn metadata_match_keeps_game_detail_background_results() {
    let mut app = app_with_library(vec![platform(1, "NES", 1)]);
    let previous = LibraryBrowseScreen::new(
        vec![platform(1, "NES", 1)],
        vec![],
        LIBRARY_LEFT_PANEL_PERCENT_DEFAULT,
    );
    let mut detail = GameDetailScreen::new(
        rom_fixture(),
        Vec::new(),
        GameDetailPrevious::Library(Box::new(previous)),
        app.downloads.shared(),
        COVER_PANEL_WIDTH_DEFAULT,
    );
    detail.saves_state = SaveListState::Loading;
    detail.achievements_state = AchievementListState::Loading;
    app.screen =
        AppScreen::MetadataMatch(Box::new(MetadataMatchScreen::new_for_rom(Box::new(detail))));

    app.apply_background(BackgroundAction::SaveList(SaveListDone {
        rom_id: 10,
        result: Ok(Vec::new()),
    }));
    app.apply_background(BackgroundAction::AchievementLoad(AchievementLoadDone {
        gen: 0,
        rom_id: 10,
        result: Ok(romm_api::core::achievements::AchievementLoadResult::Empty(
            "No achievements".into(),
        )),
    }));

    match &app.screen {
        AppScreen::MetadataMatch(picker) => {
            assert!(matches!(
                picker.previous.saves_state,
                SaveListState::Loaded(ref saves) if saves.is_empty()
            ));
            assert!(matches!(
                picker.previous.achievements_state,
                AchievementListState::Empty(ref message) if message == "No achievements"
            ));
        }
        _ => panic!("expected metadata match"),
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

struct IsolatedConfigDir {
    _guard: MutexGuard<'static, ()>,
    dir: PathBuf,
}

impl IsolatedConfigDir {
    fn new(prefix: &str) -> Self {
        let guard = romm_api::config::test_env_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("romm-tui-{prefix}-test-{ts}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("ROMM_TEST_CONFIG_DIR", &dir);
        Self { _guard: guard, dir }
    }
}

impl Drop for IsolatedConfigDir {
    fn drop(&mut self) {
        std::env::remove_var("ROMM_TEST_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&self.dir);
    }
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
async fn settings_reset_prevents_later_config_persistence() {
    let _env = IsolatedConfigDir::new("settings-reset");
    let mut app = app_on_library();

    romm_api::config::persist_user_config(&app.config).expect("seed config");
    let config_path = romm_api::config::user_config_json_path().expect("config path");
    assert!(config_path.exists(), "test setup should create config.json");

    app.screen = AppScreen::Settings(Box::new(SettingsScreen::new(
        &app.config,
        None,
        supported_save_sync_compatibility(),
    )));
    if let AppScreen::Settings(settings) = &mut app.screen {
        settings.confirm = Some(SettingsConfirm::Reset);
    }

    app.handle_key_event(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
        .await
        .expect("confirm reset");
    assert!(!config_path.exists(), "reset should remove config.json");

    app.handle_key_event(&KeyEvent::new(KeyCode::Char('S'), KeyModifiers::empty()))
        .await
        .expect("save after reset");
    assert!(
        !config_path.exists(),
        "settings save must not recreate config.json after reset"
    );

    app.persist_tui_layout();
    assert!(
        !config_path.exists(),
        "layout persistence must not recreate config.json after reset"
    );
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
