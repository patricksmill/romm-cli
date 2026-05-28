use std::collections::HashMap;

use super::types::{AUTH_MAINT_ROWS, APPEARANCE_ROWS, CONNECTION_ROWS, EXTRAS_ROWS, SAVES_ROWS};
use super::{SettingsConfirm, SettingsRow, SettingsScreen, SettingsTab};
use crate::config::{
    Config, ExtrasDefaults, RomsLayoutConfig, SaveSyncConfig, default_theme_id,
};
use crate::feature_compat::{
    supported_save_sync_compatibility, FeatureCompatibility, RequiredEndpoint, SAVE_SYNC_FEATURE,
    SAVE_SYNC_UNSUPPORTED_MESSAGE,
};

fn test_config() -> Config {
    Config {
        base_url: "https://romm.example.com".to_string(),
        download_dir: "C:\\roms".to_string(),
        use_https: true,
        auth: None,
        extras_defaults: ExtrasDefaults::default(),
        save_sync: SaveSyncConfig {
            save_dir: Some("C:\\saves".to_string()),
            device_id: None,
            platform_dirs: HashMap::new(),
        },
        roms_layout: RomsLayoutConfig::default(),
        theme: default_theme_id(),
    }
}

fn screen() -> SettingsScreen {
    SettingsScreen::new(
        &test_config(),
        Some("1.0.0"),
        supported_save_sync_compatibility(),
    )
}

fn unsupported_screen() -> SettingsScreen {
    SettingsScreen::new(
        &test_config(),
        Some("1.0.0"),
        FeatureCompatibility::from_registry(
            SAVE_SYNC_FEATURE,
            SAVE_SYNC_UNSUPPORTED_MESSAGE,
            &[RequiredEndpoint {
                method: "GET",
                path: "/api/devices",
            }],
            &crate::openapi::EndpointRegistry::default(),
        ),
    )
}

#[test]
fn tabs_expose_expected_rows() {
    let s = screen();
    assert_eq!(
        s.visible_rows(),
        vec![SettingsRow::BaseUrl, SettingsRow::UseHttps]
    );
    assert_eq!(
        CONNECTION_ROWS,
        [SettingsRow::BaseUrl, SettingsRow::UseHttps]
    );
    assert_eq!(SettingsTab::Saves as usize, SettingsTab::Roms.index() + 1);
    assert_eq!(
        SAVES_ROWS,
        [
            SettingsRow::SaveDir,
            SettingsRow::SaveConsolePaths,
            SettingsRow::SyncDevice,
            SettingsRow::SyncNow
        ]
    );
    assert_eq!(
        EXTRAS_ROWS,
        [
            SettingsRow::ExtrasRelatedRoms,
            SettingsRow::ExtrasCover,
            SettingsRow::ExtrasManual
        ]
    );
    assert_eq!(APPEARANCE_ROWS, [SettingsRow::Theme]);
    assert_eq!(
        AUTH_MAINT_ROWS,
        [
            SettingsRow::Auth,
            SettingsRow::ClearCache,
            SettingsRow::ResetConfiguration
        ]
    );
}

#[test]
fn appearance_tab_has_theme_row() {
    let mut s = screen();
    s.selected_tab = SettingsTab::Appearance;
    assert_eq!(s.visible_rows(), vec![SettingsRow::Theme]);
}

#[test]
fn theme_row_cycles_with_helpers() {
    let mut s = screen();
    s.selected_tab = SettingsTab::Appearance;
    let initial = s.theme_id.clone();
    s.cycle_theme_next();
    assert_ne!(s.theme_id, initial);
    s.cycle_theme_prev();
    assert_eq!(s.theme_id, initial);
}

#[test]
fn row_navigation_wraps_within_active_tab() {
    let mut s = screen();

    assert_eq!(s.selected_row(), SettingsRow::BaseUrl);
    s.previous();
    assert_eq!(s.selected_row(), SettingsRow::UseHttps);
    s.next();
    assert_eq!(s.selected_row(), SettingsRow::BaseUrl);
}

#[test]
fn saves_tab_shows_save_console_paths_row() {
    let mut s = screen();
    s.selected_tab = SettingsTab::Saves;
    assert_eq!(
        s.visible_rows(),
        vec![
            SettingsRow::SaveDir,
            SettingsRow::SaveConsolePaths,
            SettingsRow::SyncDevice,
            SettingsRow::SyncNow,
        ]
    );
    s.next();
    assert_eq!(s.selected_row(), SettingsRow::SaveConsolePaths);
}

#[test]
fn roms_tab_always_shows_console_paths_row() {
    let mut s = screen();
    s.selected_tab = SettingsTab::Roms;
    assert_eq!(
        s.visible_rows(),
        vec![SettingsRow::RomsDir, SettingsRow::ConsolePaths]
    );
    s.next();
    assert_eq!(s.selected_row(), SettingsRow::ConsolePaths);
}

#[test]
fn tab_navigation_preserves_per_tab_selection() {
    let mut s = screen();

    s.next();
    assert_eq!(s.selected_row(), SettingsRow::UseHttps);

    s.next_tab();
    assert_eq!(s.selected_tab, SettingsTab::Roms);
    assert_eq!(s.selected_row(), SettingsRow::RomsDir);

    s.next_tab();
    assert_eq!(s.selected_tab, SettingsTab::Saves);
    assert_eq!(s.selected_row(), SettingsRow::SaveDir);

    s.next();
    assert_eq!(s.selected_row(), SettingsRow::SaveConsolePaths);

    s.next();
    assert_eq!(s.selected_row(), SettingsRow::SyncDevice);

    s.previous_tab();
    assert_eq!(s.selected_tab, SettingsTab::Roms);
    assert_eq!(s.selected_row(), SettingsRow::RomsDir);

    s.previous_tab();
    assert_eq!(s.selected_tab, SettingsTab::Connection);
    assert_eq!(s.selected_row(), SettingsRow::UseHttps);

    s.next_tab();
    assert_eq!(s.selected_tab, SettingsTab::Roms);
    assert_eq!(s.selected_row(), SettingsRow::RomsDir);

    s.next_tab();
    assert_eq!(s.selected_tab, SettingsTab::Saves);
    assert_eq!(s.selected_row(), SettingsRow::SyncDevice);
}

#[test]
fn activation_rows_resolve_to_expected_intents() {
    let mut s = screen();

    s.selected_tab = SettingsTab::AuthMaintenance;
    assert_eq!(s.selected_row(), SettingsRow::Auth);

    s.next();
    assert_eq!(s.selected_row(), SettingsRow::ClearCache);
    s.enter_edit();
    assert_eq!(s.confirm, Some(SettingsConfirm::ClearCache));

    s.cancel_edit();
    s.next();
    assert_eq!(s.selected_row(), SettingsRow::ResetConfiguration);
    s.enter_edit();
    assert_eq!(s.confirm, Some(SettingsConfirm::Reset));
}

#[test]
fn save_action_rows_trigger_matching_state() {
    let mut s = screen();
    s.selected_tab = SettingsTab::Saves;

    s.next();
    s.next();
    assert_eq!(s.selected_row(), SettingsRow::SyncDevice);
    s.enter_edit();
    assert!(s.device_picker_open);
    assert!(s.device_picker_loading);

    s.device_picker_open = false;
    s.device_picker_loading = false;
    s.next();
    assert_eq!(s.selected_row(), SettingsRow::SyncNow);
    s.enter_edit();
    assert_eq!(
        s.message.as_ref().map(|(msg, _)| msg.as_str()),
        Some("Starting save sync...")
    );
}

#[test]
fn extras_rows_toggle_matching_defaults() {
    let mut s = screen();
    s.selected_tab = SettingsTab::Extras;

    s.enter_edit();
    assert!(!s.extras_include_related_roms);
    assert!(s.extras_include_cover);
    assert!(s.extras_include_manual);

    s.next();
    s.enter_edit();
    assert!(!s.extras_include_cover);
    assert!(s.extras_include_manual);

    s.next();
    s.enter_edit();
    assert!(!s.extras_include_manual);
}

#[test]
fn unsupported_save_sync_rows_do_not_open_device_picker_or_start_sync() {
    let mut s = unsupported_screen();
    s.selected_tab = SettingsTab::Saves;

    s.next();
    s.next();
    assert_eq!(s.selected_row(), SettingsRow::SyncDevice);
    s.enter_edit();
    assert!(!s.device_picker_open);
    assert_eq!(
            s.message.as_ref().map(|(msg, _)| msg.as_str()),
            Some(
                "This RomM server does not expose save-sync endpoints; upgrade RomM to use romm-cli sync. Missing endpoint(s): GET /api/devices"
            )
        );

    s.next();
    assert_eq!(s.selected_row(), SettingsRow::SyncNow);
    s.enter_edit();
    assert!(!s.sync_inflight);
    assert!(s
        .message
        .as_ref()
        .map(|(msg, _)| msg.contains(SAVE_SYNC_UNSUPPORTED_MESSAGE))
        .unwrap_or(false));
}

#[test]
fn unsupported_save_sync_rows_render_requires_newer_server_annotation() {
    let s = unsupported_screen();

    assert!(s
        .row_label(SettingsRow::SyncDevice)
        .contains("requires newer RomM server"));
    assert!(s
        .row_label(SettingsRow::SyncNow)
        .contains("requires newer RomM server"));
}
