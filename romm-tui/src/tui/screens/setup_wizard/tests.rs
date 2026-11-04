use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::tui::path_picker::{PathPicker, PathPickerMode};
use crate::tui::theme::{resolve_theme_or_default, RommStyles};
use romm_api::config::{default_theme_id, normalize_romm_origin, AuthConfig};
use romm_api::core::download::validate_configured_download_directory;

use super::types::{AuthKind, Step};
use super::SetupWizard;

fn unique_test_download_dir() -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("romm-dl-test-{}-{suffix}", std::process::id()))
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

/// Renders the wizard into a `TestBackend` and returns the visible screen text.
fn render_wizard(wizard: &mut SetupWizard) -> String {
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
    let buffer = terminal.backend().buffer();
    let width = buffer.area.width as usize;
    let mut screen = String::new();
    for (index, cell) in buffer.content().iter().enumerate() {
        if width > 0 && index > 0 && index % width == 0 {
            screen.push('\n');
        }
        screen.push_str(cell.symbol());
    }
    screen
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

#[test]
fn url_input_keeps_cursor_on_utf8_boundaries() {
    let mut wizard = SetupWizard::new();
    wizard.url.clear();
    wizard.url_cursor = 0;

    wizard.handle_key(&key(KeyCode::Char('é'))).unwrap();
    wizard.handle_key(&key(KeyCode::Char('x'))).unwrap();
    assert_eq!(wizard.url, "éx");

    wizard.handle_key(&key(KeyCode::Backspace)).unwrap();
    assert_eq!(wizard.url, "é");
    assert_eq!(wizard.url_cursor, wizard.url.len());

    wizard.url = "aé".to_string();
    wizard.url_cursor = wizard.url.len();
    wizard.handle_key(&key(KeyCode::Left)).unwrap();
    wizard.handle_key(&key(KeyCode::Char('x'))).unwrap();
    assert_eq!(wizard.url, "axé");
}

#[test]
fn url_renders_inline_caret_at_utf8_cursor() {
    let mut wizard = SetupWizard::new();
    wizard.url = "éx".to_string();
    wizard.url_cursor = wizard.url.len();
    assert!(render_wizard(&mut wizard).contains("éx▏"));

    wizard.url = "aé".to_string();
    wizard.url_cursor = "a".len();
    assert!(render_wizard(&mut wizard).contains("a▏é"));
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
fn hidden_password_renders_mask_with_inline_caret() {
    let mut wizard = SetupWizard::new();
    wizard.step = Step::BasicPass;
    wizard.password = "secret".to_string();
    let screen = render_wizard(&mut wizard);
    assert!(
        screen.contains("••••••▏"),
        "active password field should mask the secret and show an inline caret at the end"
    );
    assert!(
        !screen.contains("secret"),
        "password text must never be rendered"
    );
}

#[test]
fn hidden_password_mask_counts_utf8_chars_not_bytes() {
    let mut wizard = SetupWizard::new();
    wizard.step = Step::BasicPass;
    wizard.password = "é".to_string();
    let screen = render_wizard(&mut wizard);
    assert_eq!(screen.matches('•').count(), 1);
    assert!(screen.contains("•▏"));
}

#[test]
fn hidden_api_key_renders_mask_with_inline_caret_at_middle_cursor() {
    let mut wizard = SetupWizard::new();
    wizard.step = Step::ApiKey;
    wizard.api_key = "abcd".to_string();
    wizard.api_key_cursor = wizard.api_key.len();
    wizard.handle_key(&key(KeyCode::Left)).unwrap();
    wizard.handle_key(&key(KeyCode::Left)).unwrap();
    wizard.handle_key(&key(KeyCode::Char('X'))).unwrap();
    assert_eq!(wizard.api_key, "abXcd");
    assert_eq!(wizard.api_key_cursor, 3);

    let screen = render_wizard(&mut wizard);
    assert!(
        screen.contains("•••▏••"),
        "active API key field should place the caret at the edited character position"
    );
    assert!(
        !screen.contains("abXcd"),
        "API key text must never be rendered"
    );
}
