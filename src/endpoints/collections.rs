use crate::types::Collection;

use super::Endpoint;

/// List collections (smart/virtual). ROMM API: GET /api/collections.
#[derive(Debug, Default, Clone)]
pub struct ListCollections;

impl Endpoint for ListCollections {
    type Output = Vec<Collection>;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/collections".into()
    }
}
