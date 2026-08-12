//! Collection list, get, and delete helpers.

use serde_json::Value;

use crate::client::RommClient;
use crate::endpoints::collections::{
    merge_all_collection_sources, DeleteManualCollection, DeleteSmartCollection,
    GetManualCollection, GetSmartCollection, GetVirtualCollection, ListCollections,
    ListSmartCollections, ListVirtualCollections,
};
use crate::error::ApiError;
use crate::types::Collection;

/// Manual, smart, virtual, or merged library view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionKind {
    Manual,
    Smart,
    Virtual,
    All,
}

impl CollectionKind {
    pub fn parse(s: &str) -> Result<Self, ApiError> {
        match s.to_ascii_lowercase().as_str() {
            "manual" => Ok(Self::Manual),
            "smart" => Ok(Self::Smart),
            "virtual" => Ok(Self::Virtual),
            "all" => Ok(Self::All),
            _ => Err(ApiError::UnexpectedResponse(format!(
                "invalid collection type {s:?} (use manual, smart, virtual, or all)"
            ))),
        }
    }
}

pub async fn list_collections(
    client: &RommClient,
    kind: CollectionKind,
) -> Result<Vec<Collection>, ApiError> {
    match kind {
        CollectionKind::Manual => Ok(client.call(&ListCollections).await?.into_vec()),
        CollectionKind::Smart => Ok(client.call(&ListSmartCollections).await?.into_vec()),
        CollectionKind::Virtual => {
            let rows = client.call(&ListVirtualCollections).await?;
            Ok(merge_all_collection_sources(Vec::new(), Vec::new(), rows))
        }
        CollectionKind::All => {
            let manual = client.call(&ListCollections).await?.into_vec();
            let smart = client.call(&ListSmartCollections).await?.into_vec();
            let virtual_rows = client.call(&ListVirtualCollections).await?;
            Ok(merge_all_collection_sources(manual, smart, virtual_rows))
        }
    }
}

pub async fn get_collection(
    client: &RommClient,
    kind: CollectionKind,
    id: &str,
) -> Result<Value, ApiError> {
    match kind {
        CollectionKind::Manual => {
            let numeric = id.parse::<u64>().map_err(|_| {
                ApiError::UnexpectedResponse(format!("manual collection id must be numeric: {id}"))
            })?;
            client.call(&GetManualCollection { id: numeric }).await
        }
        CollectionKind::Smart => {
            let numeric = id.parse::<u64>().map_err(|_| {
                ApiError::UnexpectedResponse(format!("smart collection id must be numeric: {id}"))
            })?;
            client.call(&GetSmartCollection { id: numeric }).await
        }
        CollectionKind::Virtual | CollectionKind::All => {
            client
                .call(&GetVirtualCollection { id: id.to_string() })
                .await
        }
    }
}

pub async fn delete_collection(
    client: &RommClient,
    kind: CollectionKind,
    id: u64,
) -> Result<Value, ApiError> {
    match kind {
        CollectionKind::Manual => client.call(&DeleteManualCollection { id }).await,
        CollectionKind::Smart => client.call(&DeleteSmartCollection { id }).await,
        CollectionKind::Virtual | CollectionKind::All => Err(ApiError::UnexpectedResponse(
            "virtual collections cannot be deleted via API".into(),
        )),
    }
}
