//! Save list, download, and upload helpers (CLI + TUI parity).

use std::path::{Path, PathBuf};

use crate::client::{RommClient, SaveUploadOptions};
use crate::endpoints::saves::SaveSchema;
use crate::endpoints::saves::{GetSave, ListSaves};
use crate::error::ApiError;
use crate::types::SaveMetadata;

/// Filters for `GET /api/saves`.
#[derive(Debug, Clone, Default)]
pub struct SaveListFilter {
    pub rom_id: Option<u64>,
    pub device_id: Option<String>,
    pub slot: Option<String>,
}

/// List saves with full metadata (same parsing as the TUI Saves tab).
pub async fn list_saves(
    client: &RommClient,
    filter: SaveListFilter,
) -> Result<Vec<SaveMetadata>, ApiError> {
    let mut query: Vec<(String, String)> = Vec::new();
    if let Some(rom_id) = filter.rom_id {
        query.push(("rom_id".into(), rom_id.to_string()));
    }
    if let Some(ref device_id) = filter.device_id {
        if !device_id.is_empty() {
            query.push(("device_id".into(), device_id.clone()));
        }
    }
    if let Some(ref slot) = filter.slot {
        if !slot.is_empty() {
            query.push(("slot".into(), slot.clone()));
        }
    }
    let value = client
        .request_json("GET", "/api/saves", &query, None)
        .await?;
    SaveMetadata::from_api_value(value).map_err(|e| ApiError::UnexpectedResponse(e.to_string()))
}

/// `GET /api/saves/{id}`.
pub async fn get_save(
    client: &RommClient,
    id: u64,
    device_id: Option<&str>,
) -> Result<SaveSchema, ApiError> {
    client
        .call(&GetSave {
            id,
            device_id: device_id.map(str::to_string),
        })
        .await
}

/// Download save bytes to `dest` (creates parent directories when needed).
pub async fn download_save_to_path(
    client: &RommClient,
    save_id: u64,
    dest: &Path,
    device_id: Option<&str>,
    session_id: Option<u64>,
) -> Result<PathBuf, ApiError> {
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    let bytes = client
        .download_save_content(save_id, device_id, session_id)
        .await?;
    tokio::fs::write(dest, &bytes).await?;
    Ok(dest.to_path_buf())
}

/// Upload a local save file for a ROM.
pub async fn upload_save_for_rom(
    client: &RommClient,
    rom_id: u64,
    path: &Path,
    options: SaveUploadOptions<'_>,
) -> Result<serde_json::Value, ApiError> {
    client
        .upload_save_file_with_options(rom_id, path, &options)
        .await
}

/// List saves via typed endpoint (minimal schema).
pub async fn list_saves_typed(
    client: &RommClient,
    filter: SaveListFilter,
) -> Result<Vec<SaveSchema>, ApiError> {
    client
        .call(&ListSaves {
            rom_id: filter.rom_id,
            device_id: filter.device_id,
            slot: filter.slot,
        })
        .await
}
