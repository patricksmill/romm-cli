//! Terminal UI module.

pub mod app;
pub mod cache;
pub mod download;
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

    let mut app = App::new(client, config, registry);
    app.run().await
}
