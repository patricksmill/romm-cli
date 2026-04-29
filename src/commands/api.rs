use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};

use crate::client::RommClient;
use crate::commands::OutputFormat;

/// Low-level escape hatch for calling arbitrary ROMM API endpoints.
#[derive(Args, Debug)]
pub struct ApiCommand {
    #[command(subcommand)]
    pub action: Option<ApiAction>,

    /// HTTP method (legacy, use 'api call `<method>` `<path>`')
    pub method: Option<String>,

    /// API path (legacy, use 'api call `<method>` `<path>`')
    pub path: Option<String>,

    /// Query parameters as key=value, repeatable
    #[arg(long = "query", global = true)]
    pub query: Vec<String>,

    /// JSON request body as a string
    #[arg(long, global = true)]
    pub data: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ApiAction {
    /// Make a generic API call
    Call {
        /// HTTP method (GET, POST, etc.)
        method: String,
        /// API path (e.g. /api/roms)
        path: String,
    },
    /// Shortcut for GET request
    Get {
        /// API path
        path: String,
    },
    /// Shortcut for POST request
    Post {
        /// API path
        path: String,
    },
}

pub async fn handle(cmd: ApiCommand, client: &RommClient, format: OutputFormat) -> Result<()> {
    let (method, path) = match cmd.action {
        Some(ApiAction::Call { method, path }) => (method, path),
        Some(ApiAction::Get { path }) => ("GET".to_string(), path),
        Some(ApiAction::Post { path }) => ("POST".to_string(), path),
        None => {
            let m = cmd
                .method
                .ok_or_else(|| anyhow!("Method is required (e.g. 'api call GET /api/roms')"))?;
            let p = cmd
                .path
                .ok_or_else(|| anyhow!("Path is required (e.g. 'api call GET /api/roms')"))?;
            (m, p)
        }
    };

    let mut query_pairs = Vec::new();
    for q in &cmd.query {
        if let Some((k, v)) = q.split_once('=') {
            query_pairs.push((k.to_string(), v.to_string()));
        } else {
            eprintln!(
                "warning: ignoring malformed --query value {:?}; expected key=value",
                q
            );
        }
    }

    let body = if let Some(data) = &cmd.data {
        Some(serde_json::from_str(data)?)
    } else {
        None
    };

    let value = client
        .request_json(&method, &path, &query_pairs, body)
        .await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        OutputFormat::Text => {
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
    }
    Ok(())
}
