//! Save sync session endpoints (`/api/sync/*`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Endpoint;

/// One operation returned by `POST /api/sync/negotiate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub action: String,
    pub rom_id: u64,
    pub save_id: Option<u64>,
    pub file_name: String,
    pub slot: Option<String>,
    pub emulator: Option<String>,
    pub reason: String,
    pub server_updated_at: Option<String>,
    pub server_content_hash: Option<String>,
}

/// `POST /api/sync/negotiate` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncNegotiateResponse {
    pub session_id: u64,
    pub operations: Vec<SyncOperation>,
    pub total_upload: u64,
    pub total_download: u64,
    pub total_conflict: u64,
    pub total_no_op: u64,
}

/// `GET /api/sync/sessions*` response row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSessionSchema {
    pub id: u64,
    pub device_id: String,
    pub user_id: u64,
    pub status: String,
    pub initiated_at: String,
    pub completed_at: Option<String>,
    pub operations_planned: u64,
    pub operations_completed: u64,
    pub operations_failed: u64,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCompleteResponse {
    pub session: SyncSessionSchema,
    pub play_session_ingest: Option<Value>,
}

/// `POST /api/sync/negotiate`.
#[derive(Debug, Clone)]
pub struct NegotiateSync {
    pub body: Value,
}

impl Endpoint for NegotiateSync {
    type Output = SyncNegotiateResponse;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        "/api/sync/negotiate".into()
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `POST /api/sync/sessions/{session_id}/complete`.
#[derive(Debug, Clone)]
pub struct CompleteSyncSession {
    pub session_id: u64,
    pub body: Value,
}

impl Endpoint for CompleteSyncSession {
    type Output = SyncCompleteResponse;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        format!("/api/sync/sessions/{}/complete", self.session_id)
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `GET /api/sync/sessions`.
#[derive(Debug, Clone, Default)]
pub struct ListSyncSessions {
    pub device_id: Option<String>,
    pub limit: Option<u32>,
}

impl Endpoint for ListSyncSessions {
    type Output = Vec<SyncSessionSchema>;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/sync/sessions".into()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(ref device_id) = self.device_id {
            if !device_id.is_empty() {
                out.push(("device_id".into(), device_id.clone()));
            }
        }
        if let Some(limit) = self.limit {
            out.push(("limit".into(), limit.to_string()));
        }
        out
    }
}

/// `GET /api/sync/sessions/{session_id}`.
#[derive(Debug, Clone)]
pub struct GetSyncSession {
    pub session_id: u64,
}

impl Endpoint for GetSyncSession {
    type Output = SyncSessionSchema;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        format!("/api/sync/sessions/{}", self.session_id)
    }
}

/// `POST /api/sync/devices/{device_id}/push-pull`.
#[derive(Debug, Clone)]
pub struct TriggerPushPull {
    pub device_id: String,
}

impl Endpoint for TriggerPushPull {
    type Output = SyncSessionSchema;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        format!("/api/sync/devices/{}/push-pull", self.device_id)
    }
}
