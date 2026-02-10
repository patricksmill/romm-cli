//! Small service objects that wrap `RommClient` for higher-level operations.
//!
//! These are used by the CLI commands to keep a clear separation between
//! \"how we talk to ROMM\" (HTTP) and \"what we want to do\" (list
//! platforms, search ROMs, etc.).

use anyhow::Result;
use async_trait::async_trait;

use crate::client::RommClient;
use crate::endpoints::{platforms::ListPlatforms, roms::GetRoms};
use crate::types::{Platform, RomList};

/// Async API for platform-related operations.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PlatformApi {
    async fn list_platforms(&self) -> Result<Vec<Platform>>;
}

/// Service encapsulating platform-related operations.
pub struct PlatformService<'a> {
    client: &'a RommClient,
}

impl<'a> PlatformService<'a> {
    pub fn new(client: &'a RommClient) -> Self {
        Self { client }
    }

    /// List all platforms from the ROMM API.
    pub async fn list_platforms(&self) -> Result<Vec<Platform>> {
        let platforms = self.client.call(&ListPlatforms::default()).await?;
        Ok(platforms)
    }
}

#[async_trait]
impl<'a> PlatformApi for PlatformService<'a> {
    async fn list_platforms(&self) -> Result<Vec<Platform>> {
        self.list_platforms().await
    }
}

/// Async API for ROM-related operations.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RomApi {
    async fn search_roms(&self, ep: &GetRoms) -> Result<RomList>;
}

/// Service encapsulating ROM-related operations.
pub struct RomService<'a> {
    client: &'a RommClient,
}

impl<'a> RomService<'a> {
    pub fn new(client: &'a RommClient) -> Self {
        Self { client }
    }

    /// Search/list ROMs using a fully-configured `GetRoms` descriptor.
    pub async fn search_roms(&self, ep: &GetRoms) -> Result<RomList> {
        let results = self.client.call(ep).await?;
        Ok(results)
    }
}

#[async_trait]
impl<'a> RomApi for RomService<'a> {
    async fn search_roms(&self, ep: &GetRoms) -> Result<RomList> {
        self.search_roms(ep).await
    }
}


