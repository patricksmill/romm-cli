use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use romm_cli::tui::app::{App, AppScreen};
use romm_cli::client::RommClient;
use romm_cli::config::Config;
use romm_cli::tui::openapi::EndpointRegistry;
use crossterm::event::KeyCode;

#[tokio::test]
async fn test_main_menu_api_error_shows_popup() {
    let mock_server = MockServer::start().await;
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
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None);

    // Simulate pressing Enter on Main Menu (Platforms)
    let quit = app.handle_key(KeyCode::Enter).await.unwrap();
    assert!(!quit);
    
    // Assert error is set and we didn't crash
    assert!(app.global_error.is_some());
    assert!(app.global_error.as_ref().unwrap().contains("500"));
    
    // Assert Esc clears it
    app.handle_key(KeyCode::Esc).await.unwrap();
    assert!(app.global_error.is_none());
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

    let config = Config {
        base_url: mock_server.uri(),
        download_dir: "/tmp".into(),
        use_https: false,
        auth: None,
    };
    let client = RommClient::new(&config, false).unwrap();
    let mut app = App::new(client, config, EndpointRegistry::default(), None, None);

    // Simulate pressing Enter on Main Menu (Platforms)
    let quit = app.handle_key(KeyCode::Enter).await.unwrap();
    assert!(!quit);
    
    // Assert error is not set
    assert!(app.global_error.is_none());
    
    // Assert we transitioned to LibraryBrowse
    assert!(matches!(app.screen, AppScreen::LibraryBrowse(_)));
}
