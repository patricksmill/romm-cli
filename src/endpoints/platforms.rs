use crate::types::Platform;

use super::Endpoint;

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
