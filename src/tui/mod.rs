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
pub mod openapi_sync;
pub mod screens;
pub mod utils;

use anyhow::Result;

use crate::client::RommClient;
use crate::config::{openapi_cache_path, Config};

use self::app::App;
use self::openapi_sync::sync_openapi_registry;

/// Launch the interactive TUI.
pub async fn run(client: RommClient, config: Config) -> Result<()> {
    let cache_path = openapi_cache_path()?;
    let (registry, server_version) = sync_openapi_registry(&client, &cache_path).await?;

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

    let mut app = App::new(client, config, registry, server_version);
    app.run().await
}
