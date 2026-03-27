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
pub mod openapi;
pub mod screens;
pub mod utils;

use anyhow::{anyhow, Result};

use crate::client::RommClient;
use crate::config::Config;

use self::app::App;
use self::openapi::EndpointRegistry;

/// Launch the interactive TUI.
pub async fn run(client: RommClient, config: Config) -> Result<()> {
    let openapi_path =
        std::env::var("ROMM_OPENAPI_PATH").unwrap_or_else(|_| "openapi.json".to_string());

    let registry = if std::path::Path::new(&openapi_path).exists() {
        EndpointRegistry::from_file(&openapi_path)?
    } else {
        return Err(anyhow!(
            "OpenAPI file not found at {}. Set ROMM_OPENAPI_PATH or provide openapi.json.",
            openapi_path
        ));
    };

    // Ensure terminal is cleaned up if a panic occurs.
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

    let mut app = App::new(client, config, registry);
    app.run().await
}
