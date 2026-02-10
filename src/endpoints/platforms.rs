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
