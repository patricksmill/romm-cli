//! System and authentication helper endpoints (heartbeat, stats, profile).

use serde_json::Value;

use super::Endpoint;

/// `GET /api/heartbeat` — server metadata (authenticated when credentials are set).
#[derive(Debug, Default, Clone)]
pub struct GetHeartbeat;

impl Endpoint for GetHeartbeat {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/heartbeat".into()
    }
}

/// `GET /api/stats` — aggregate library statistics.
#[derive(Debug, Default, Clone)]
pub struct GetStats;

impl Endpoint for GetStats {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/stats".into()
    }
}

/// `GET /api/users/me` — current user profile.
#[derive(Debug, Default, Clone)]
pub struct GetUsersMe;

impl Endpoint for GetUsersMe {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/users/me".into()
    }
}
