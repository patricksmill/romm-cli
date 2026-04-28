use crate::types::Platform;

use super::Endpoint;
use serde_json::Value;

/// List all platforms.
#[derive(Debug, Default, Clone)]
pub struct ListPlatforms;

impl Endpoint for ListPlatforms {
    type Output = Vec<Platform>;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/platforms".into()
    }
}

/// Retrieve a platform by ID.
#[derive(Debug, Clone)]
pub struct GetPlatform {
    pub id: u64,
}

impl Endpoint for GetPlatform {
    type Output = Platform;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        format!("/api/platforms/{}", self.id)
    }
}

/// `GET /api/platforms/supported` — IGDB-supported platform catalog.
#[derive(Debug, Default, Clone)]
pub struct ListSupportedPlatforms;

impl Endpoint for ListSupportedPlatforms {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/platforms/supported".into()
    }
}

/// `PUT /api/platforms/{id}` — update platform metadata (JSON body).
#[derive(Debug, Clone)]
pub struct PutPlatform {
    pub id: u64,
    pub body: Value,
}

impl Endpoint for PutPlatform {
    type Output = Value;

    fn method(&self) -> &'static str {
        "PUT"
    }

    fn path(&self) -> String {
        format!("/api/platforms/{}", self.id)
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `DELETE /api/platforms/{id}`
#[derive(Debug, Clone)]
pub struct DeletePlatform {
    pub id: u64,
}

impl Endpoint for DeletePlatform {
    type Output = Value;

    fn method(&self) -> &'static str {
        "DELETE"
    }

    fn path(&self) -> String {
        format!("/api/platforms/{}", self.id)
    }
}
