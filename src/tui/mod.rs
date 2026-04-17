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
pub mod text_search;
pub mod openapi_sync;
pub mod screens;
pub mod utils;

use anyhow::Result;

use crate::client::RommClient;
use crate::config::{openapi_cache_path, Config};

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

async fn run_started(client: RommClient, config: Config, from_setup_wizard: bool) -> Result<()> {
    install_panic_hook();
    let cache_path = openapi_cache_path()?;
    let (registry, server_version) = sync_openapi_registry(&client, &cache_path).await?;

    let splash = startup_splash_for(from_setup_wizard, &config, &server_version);
    let mut app = App::new(client, config, registry, server_version, splash);
    app.run().await
}

/// Launch the TUI when the caller already has a [`RommClient`] and [`Config`].
pub async fn run(client: RommClient, config: Config) -> Result<()> {
    run_started(client, config, false).await
}

/// Load config, run first-time setup in the terminal if `API_BASE_URL` is missing, then start the TUI.
pub async fn run_interactive(verbose: bool) -> Result<()> {
    let (from_wizard, config) = match crate::config::load_config() {
        Ok(c) => (false, c),
        Err(_) => (true, SetupWizard::new().run(verbose).await?),
    };
    let client = RommClient::new(&config, verbose)?;
    run_started(client, config, from_wizard).await
}
