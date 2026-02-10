use anyhow::Result;
use clap::Args;

use crate::client::RommClient;
use crate::commands::OutputFormat;

/// Low-level escape hatch for calling arbitrary ROMM API endpoints.
///
/// This command is most similar to `curl`: you provide the method,
/// path, query parameters, and optional JSON body. Authentication and
/// base URL are still taken from the usual `Config`.
#[derive(Args, Debug)]
pub struct ApiCommand {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,

    /// API path, e.g. /api/roms
    pub path: String,

    /// Query parameters as key=value, repeatable
    #[arg(long = "query")]
    pub query: Vec<String>,

    /// JSON request body as a string
    #[arg(long)]
    pub data: Option<String>,
}

pub async fn handle(
    cmd: ApiCommand,
    client: &RommClient,
    format: OutputFormat,
) -> Result<()> {
    let mut query_pairs = Vec::new();
    for q in &cmd.query {
        if let Some((k, v)) = q.split_once('=') {
            query_pairs.push((k.to_string(), v.to_string()));
        }
    }

    let body = if let Some(data) = &cmd.data {
        Some(serde_json::from_str(data)?)
    } else {
        None
    };

    let value = client
        .request_json(&cmd.method, &cmd.path, &query_pairs, body)
        .await?;

    match format {
        OutputFormat::Json => {
            // Machine-friendly: still pretty-printed for now.
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        OutputFormat::Text => {
            // There is no domain-specific text representation here, so we
            // fall back to pretty-printed JSON.
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
    }
    Ok(())
}
