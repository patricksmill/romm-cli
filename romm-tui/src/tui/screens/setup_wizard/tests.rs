use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::config::{default_theme_id, normalize_romm_origin, AuthConfig};
use crate::core::download::validate_configured_download_directory;
use crate::tui::path_picker::{PathPicker, PathPickerMode};
use crate::tui::theme::{resolve_theme_or_default, RommStyles};

use super::types::{AuthKind, Step};
use super::SetupWizard;

fn unique_test_download_dir() -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("romm-dl-test-{}-{suffix}", std::process::id()))
}

fn wizard_with_pairing(mock_uri: &str, code: &str, download_dir: &str) -> SetupWizard {
    SetupWizard {
        step: Step::PairingCode,
        auth_kind: AuthKind::Pairing,
        auth_menu_selected: 4,
        url: mock_uri.to_string(),
        url_cursor: mock_uri.len(),
        download_picker: PathPicker::new(PathPickerMode::Directory, download_dir),
        username: String::new(),
        user_cursor: 0,
        password: String::new(),
        bearer_token: String::new(),
        bearer_cursor: 0,
        api_header: String::new(),
        header_cursor: 0,
        api_key: String::new(),
        api_key_cursor: 0,
        pairing_code: code.to_string(),
        pairing_cursor: code.len(),
        reuse_keyring_password: false,
        reuse_keyring_bearer: false,
        reuse_keyring_api_key: false,
        testing: false,
        use_https: false,
        skip_custom_console_paths: false,
        error: None,
    }
}

#[tokio::test]
async fn pairing_config_from_exchange_returns_bearer_token() {
    let mock_server = MockServer::start().await;

    let token_json = serde_json::json!({
        "id": 1,
        "name": "cli-device",
        "scopes": [],
        "expires_at": null,
        "last_used_at": null,
        "created_at": "2020-01-01T00:00:00Z",
        "user_id": 42,
        "raw_token": "exchanged-bearer-secret"
    });

    Mock::given(method("POST"))
        .and(path("/api/client-tokens/exchange"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&token_json))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let download_dir = unique_test_download_dir();
    let download_dir = download_dir.to_string_lossy().into_owned();
    let wizard = wizard_with_pairing(&uri, "ABCD1234", &download_dir);
    let cfg = wizard
        .pairing_config_from_exchange(false)
        .await
        .expect("pairing exchange should succeed");

    match cfg.auth {
        Some(AuthConfig::Bearer { token }) => {
            assert_eq!(token, "exchanged-bearer-secret");
        }
        _ => panic!("expected bearer auth after pairing exchange"),
    }
    assert_eq!(cfg.base_url, normalize_romm_origin(&uri));
    let expected_download_dir = validate_configured_download_directory(&download_dir).unwrap();
    assert_eq!(
        cfg.download_dir,
        expected_download_dir.display().to_string()
    );
}

#[test]
fn hidden_password_field_does_not_render_inline_cursor_glyph() {
    let mut wizard = SetupWizard::new();
    wizard.step = Step::BasicPass;
    wizard.password = "secret".to_string();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    let theme = resolve_theme_or_default(&default_theme_id());
    let styles = RommStyles::new(theme.as_ref());
    terminal
        .draw(|frame| {
            let area = frame.area();
            wizard.render(frame, area, &styles);
        })
        .expect("render setup wizard");
    let backend = terminal.backend();
    let buffer = backend.buffer();
    let has_cursor_glyph = buffer.content().iter().any(|cell| cell.symbol() == "▏");
    assert!(
        !has_cursor_glyph,
        "password field should rely on terminal cursor, not inline glyph"
    );
}

#[test]
fn hidden_api_key_field_does_not_render_inline_cursor_glyph() {
    let mut wizard = SetupWizard::new();
    wizard.step = Step::ApiKey;
    wizard.api_key = "secret-key".to_string();
    wizard.api_key_cursor = wizard.api_key.len();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    let theme = resolve_theme_or_default(&default_theme_id());
    let styles = RommStyles::new(theme.as_ref());
    terminal
        .draw(|frame| {
            let area = frame.area();
            wizard.render(frame, area, &styles);
        })
        .expect("render setup wizard");
    let backend = terminal.backend();
    let buffer = backend.buffer();
    let has_cursor_glyph = buffer.content().iter().any(|cell| cell.symbol() == "▏");
    assert!(
        !has_cursor_glyph,
        "API key field should rely on terminal cursor, not inline glyph"
    );
}
