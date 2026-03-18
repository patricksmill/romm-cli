//! Small service objects that wrap `RommClient` for higher-level operations.
//!
//! These are used by the CLI commands to keep a clear separation between
//! \"how we talk to ROMM\" (HTTP) and \"what we want to do\" (list
//! platforms, search ROMs, etc.).

use anyhow::Result;

use crate::client::RommClient;
use crate::endpoints::{platforms::ListPlatforms, roms::GetRoms};
use crate::types::{Platform, RomList};

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
        let platforms = self.client.call(&ListPlatforms).await?;
        Ok(platforms)
    }
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
