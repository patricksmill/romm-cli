use crate::types::RomList;

use super::Endpoint;

/// Retrieve ROMs with optional filters.
#[derive(Debug, Default, Clone)]
pub struct GetRoms {
    pub search_term: Option<String>,
    pub platform_id: Option<u64>,
    pub collection_id: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Endpoint for GetRoms {
    type Output = RomList;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/roms".into()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();

        if let Some(term) = &self.search_term {
            q.push(("search_term".into(), term.clone()));
        }

        // RomM API expects "platform_ids" (plural); repeat param for multiple values.
        if let Some(pid) = self.platform_id {
            q.push(("platform_ids".into(), pid.to_string()));
        }

        if let Some(cid) = self.collection_id {
            q.push(("collection_id".into(), cid.to_string()));
        }

        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }

        if let Some(offset) = self.offset {
            q.push(("offset".into(), offset.to_string()));
        }

        q
    }
}
