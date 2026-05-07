//! Terminal UI module.
//!
//! This module contains all ratatui / crossterm code and is responsible
//! purely for presentation and interaction. It talks to the rest of the
//! application through:
//! - `RommClient` (HTTP / data access),
//! - `core::cache::RomCache` (disk-backed ROM cache), and
//! - `core::download::DownloadManager` (background ROM downloads).
//!
//! Keeping those \"service\" types UI-agnostic makes it easy to add other
//! frontends (e.g. a GUI) reusing the same core logic.

pub mod app;
pub mod keyboard_help;
pub mod openapi;
pub mod openapi_sync;
pub mod path_picker;
pub mod screens;
pub mod text_search;
pub mod utils;

use anyhow::Result;
use std::time::Duration;

use crate::client::RommClient;
use crate::config::{openapi_cache_path, should_check_updates, Config};

use self::app::App;
use self::openapi_sync::sync_openapi_registry;
use self::screens::connected_splash::StartupSplash;
use self::screens::setup_wizard::SetupWizard;

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        original_hook(panic);
    }));
}

fn startup_splash_for(
    from_setup_wizard: bool,
    config: &Config,
    server_version: &Option<String>,
) -> Option<StartupSplash> {
    if from_setup_wizard {
        return Some(StartupSplash::new(
            config.base_url.clone(),
            server_version.clone(),
        ));
    }
    if server_version.is_some() {
        return Some(StartupSplash::new(
            config.base_url.clone(),
            server_version.clone(),
        ));
    }
    None
}

async fn run_started(
    client: RommClient,
    config: Config,
    from_setup_wizard: bool,
    mock_update: bool,
) -> Result<()> {
    install_panic_hook();
    let cache_path = openapi_cache_path()?;
    let (registry, server_version) = sync_openapi_registry(&client, &cache_path).await?;

    let startup_update = if mock_update {
        Some(crate::update::UpdateStatus {
            current_version: format!("{} (dev)", env!("CARGO_PKG_VERSION")),
            latest_version: "9.9.9-mock".into(),
            should_update: true,
            release_url: "https://github.com/patricksmill/romm-cli".into(),
            changelog_url: crate::update::changelog_url().to_string(),
        })
    } else if should_check_updates() {
        match tokio::time::timeout(Duration::from_secs(2), crate::update::check_for_update()).await
        {
            Ok(Ok(status)) if status.should_update => Some(status),
            _ => None,
        }
    } else {
        None
    };

    let splash = startup_splash_for(from_setup_wizard, &config, &server_version);
    let mut app = App::new(
        client,
        config,
        registry,
        server_version,
        splash,
        startup_update,
    );
    app.run().await
}

/// Launch the TUI when the caller already has a [`RommClient`] and [`Config`].
pub async fn run(client: RommClient, config: Config, mock_update: bool) -> Result<()> {
    run_started(client, config, false, mock_update).await
}

/// Load config, run first-time setup in the terminal if `API_BASE_URL` is missing, then start the TUI.
pub async fn run_interactive(verbose: bool, mock_update: bool) -> Result<()> {
    let (from_wizard, config) = match crate::config::load_config() {
        Ok(c) => (false, c),
        Err(_) => (true, SetupWizard::new().run(verbose).await?),
    };
    let client = RommClient::new(&config, verbose)?;
    run_started(client, config, from_wizard, mock_update).await
}
