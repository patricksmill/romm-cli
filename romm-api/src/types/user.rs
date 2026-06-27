use serde::{Deserialize, Serialize};

use super::achievements::RaUserProgression;

/// Current authenticated user from `GET /api/users/me`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct CurrentUser {
    pub id: u64,
    #[serde(default)]
    pub ra_username: Option<String>,
    #[serde(default)]
    pub ra_progression: Option<RaUserProgression>,
}
