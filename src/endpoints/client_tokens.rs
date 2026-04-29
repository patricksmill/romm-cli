//! API endpoints for managing client tokens.
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Endpoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTokenCreateSchema {
    pub id: i64,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
    pub user_id: i64,
    pub raw_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeClientTokenRequest {
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct ExchangeClientToken {
    pub code: String,
}

impl Endpoint for ExchangeClientToken {
    type Output = ClientTokenCreateSchema;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        "/api/client-tokens/exchange".into()
    }

    fn body(&self) -> Option<Value> {
        let req = ExchangeClientTokenRequest {
            code: self.code.clone(),
        };
        serde_json::to_value(&req).ok()
    }
}
