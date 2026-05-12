//! Device management endpoints (`/api/devices`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Endpoint;

/// Sync mode expected by RomM device endpoints.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    Api,
    FileTransfer,
    PushPull,
}

/// Response from `POST /api/devices`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCreateResponse {
    pub device_id: String,
    pub name: Option<String>,
    pub created_at: String,
}

/// Response shape for `GET /api/devices*`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSchema {
    pub id: String,
    pub user_id: u64,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub client: Option<String>,
    pub client_version: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub hostname: Option<String>,
    pub sync_mode: SyncMode,
    pub sync_enabled: bool,
    pub sync_config: Option<Value>,
    pub last_seen: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// `POST /api/devices`.
#[derive(Debug, Clone)]
pub struct RegisterDevice {
    pub body: Value,
}

impl Endpoint for RegisterDevice {
    type Output = DeviceCreateResponse;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        "/api/devices".into()
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `GET /api/devices`.
#[derive(Debug, Clone, Default)]
pub struct ListDevices;

impl Endpoint for ListDevices {
    type Output = Vec<DeviceSchema>;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/devices".into()
    }
}

/// `GET /api/devices/{device_id}`.
#[derive(Debug, Clone)]
pub struct GetDevice {
    pub device_id: String,
}

impl Endpoint for GetDevice {
    type Output = DeviceSchema;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        format!("/api/devices/{}", self.device_id)
    }
}

/// `PUT /api/devices/{device_id}`.
#[derive(Debug, Clone)]
pub struct UpdateDevice {
    pub device_id: String,
    pub body: Value,
}

impl Endpoint for UpdateDevice {
    type Output = DeviceSchema;

    fn method(&self) -> &'static str {
        "PUT"
    }

    fn path(&self) -> String {
        format!("/api/devices/{}", self.device_id)
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `DELETE /api/devices/{device_id}`.
#[derive(Debug, Clone)]
pub struct DeleteDevice {
    pub device_id: String,
}

impl Endpoint for DeleteDevice {
    type Output = Value;

    fn method(&self) -> &'static str {
        "DELETE"
    }

    fn path(&self) -> String {
        format!("/api/devices/{}", self.device_id)
    }
}
