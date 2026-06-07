//! Save asset endpoints (`/api/saves*`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::Endpoint;

/// Minimal save schema used by sync command flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSchema {
    pub id: u64,
    pub rom_id: u64,
    pub file_name: String,
    pub emulator: Option<String>,
    pub slot: Option<String>,
    pub content_hash: Option<String>,
    pub updated_at: String,
}

/// `GET /api/saves`.
#[derive(Debug, Clone, Default)]
pub struct ListSaves {
    pub rom_id: Option<u64>,
    pub device_id: Option<String>,
    pub slot: Option<String>,
}

impl Endpoint for ListSaves {
    type Output = Vec<SaveSchema>;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/saves".into()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(rom_id) = self.rom_id {
            out.push(("rom_id".into(), rom_id.to_string()));
        }
        if let Some(ref device_id) = self.device_id {
            if !device_id.is_empty() {
                out.push(("device_id".into(), device_id.clone()));
            }
        }
        if let Some(ref slot) = self.slot {
            if !slot.is_empty() {
                out.push(("slot".into(), slot.clone()));
            }
        }
        out
    }
}

/// `GET /api/saves/{id}`.
#[derive(Debug, Clone)]
pub struct GetSave {
    pub id: u64,
    pub device_id: Option<String>,
}

impl Endpoint for GetSave {
    type Output = SaveSchema;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        format!("/api/saves/{}", self.id)
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(ref device_id) = self.device_id {
            if !device_id.is_empty() {
                out.push(("device_id".into(), device_id.clone()));
            }
        }
        out
    }
}

/// `POST /api/saves/{id}/downloaded`.
#[derive(Debug, Clone)]
pub struct ConfirmSaveDownloaded {
    pub id: u64,
    pub device_id: String,
}

impl Endpoint for ConfirmSaveDownloaded {
    type Output = SaveSchema;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        format!("/api/saves/{}/downloaded", self.id)
    }

    fn body(&self) -> Option<Value> {
        Some(json!({ "device_id": self.device_id }))
    }
}

/// `POST /api/saves/{id}/track`.
#[derive(Debug, Clone)]
pub struct TrackSave {
    pub id: u64,
    pub device_id: String,
}

impl Endpoint for TrackSave {
    type Output = SaveSchema;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        format!("/api/saves/{}/track", self.id)
    }

    fn body(&self) -> Option<Value> {
        Some(json!({ "device_id": self.device_id }))
    }
}

/// `POST /api/saves/{id}/untrack`.
#[derive(Debug, Clone)]
pub struct UntrackSave {
    pub id: u64,
    pub device_id: String,
}

impl Endpoint for UntrackSave {
    type Output = SaveSchema;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        format!("/api/saves/{}/untrack", self.id)
    }

    fn body(&self) -> Option<Value> {
        Some(json!({ "device_id": self.device_id }))
    }
}
