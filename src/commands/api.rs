use anyhow::Result;
use clap::Args;

use crate::client::RommClient;

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

pub async fn handle(cmd: ApiCommand, client: &RommClient) -> Result<()> {
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

    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

