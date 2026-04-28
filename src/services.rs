//! Small service objects that wrap `RommClient` for higher-level operations.
//!
//! These are used by the CLI commands to keep a clear separation between
//! \"how we talk to ROMM\" (HTTP) and \"what we want to do\" (list
//! platforms, search ROMs, etc.).

use anyhow::{anyhow, Result};

use crate::client::RommClient;
use crate::endpoints::collections::{ListCollections, ListSmartCollections};
use crate::endpoints::{
    platforms::{GetPlatform, ListPlatforms},
    roms::{GetRom, GetRoms},
};
use crate::types::{Collection, Platform, Rom, RomList};

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

    /// Get a single platform by ID.
    pub async fn get_platform(&self, id: u64) -> Result<Platform> {
        let platform = self.client.call(&GetPlatform { id }).await?;
        Ok(platform)
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

    /// Get a single ROM by ID.
    pub async fn get_rom(&self, id: u64) -> Result<Rom> {
        let rom = self.client.call(&GetRom { id }).await?;
        Ok(rom)
    }
}

/// Resolve a numeric platform ID, or match by slug / display name / custom name (same rules as the CLI).
pub fn resolve_platform_id_from_list(query: &str, platforms: &[Platform]) -> Result<u64> {
    let normalized = query.trim().to_ascii_lowercase();

    if let Some(platform) = platforms.iter().find(|p| {
        p.slug.eq_ignore_ascii_case(&normalized) || p.fs_slug.eq_ignore_ascii_case(&normalized)
    }) {
        return Ok(platform.id);
    }

    let exact_name_matches: Vec<&Platform> = platforms
        .iter()
        .filter(|p| {
            p.name.eq_ignore_ascii_case(&normalized)
                || p.display_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(&normalized))
                || p.custom_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(&normalized))
        })
        .collect();

    match exact_name_matches.len() {
        1 => Ok(exact_name_matches[0].id),
        0 => Err(anyhow!(
            "No platform found for '{}'. Use 'romm-cli platforms list' to inspect available values.",
            query
        )),
        _ => {
            let names = exact_name_matches
                .iter()
                .map(|p| format!("{} ({})", p.name, p.id))
                .collect::<Vec<_>>()
                .join(", ");
            Err(anyhow!(
                "Platform '{}' is ambiguous. Matches: {}. Please use a more specific --platform value.",
                query,
                names
            ))
        }
    }
}

/// Resolve optional platform slug/name to a single RomM `platform_ids` value.
pub async fn resolve_platform_id(
    client: &RommClient,
    platform_query: Option<&str>,
) -> Result<Option<u64>> {
    let Some(query) = platform_query.map(str::trim).filter(|q| !q.is_empty()) else {
        return Ok(None);
    };
    let service = PlatformService::new(client);
    let platforms = service.list_platforms().await?;
    resolve_platform_id_from_list(query, &platforms).map(Some)
}

/// Resolve several platform names/slugs to IDs (deduped, stable order).
pub async fn resolve_platform_ids(
    client: &RommClient,
    names: &[String],
) -> Result<Vec<u64>> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let service = PlatformService::new(client);
    let platforms = service.list_platforms().await?;
    let mut out = Vec::new();
    for n in names {
        let id = resolve_platform_id_from_list(n.trim(), &platforms)?;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    Ok(out)
}

fn match_collections_by_name<'a>(
    q: &str,
    collections: &'a [Collection],
) -> Vec<&'a Collection> {
    let n = q.trim().to_ascii_lowercase();
    collections
        .iter()
        .filter(|c| c.name.eq_ignore_ascii_case(&n))
        .collect()
}

/// Resolve manual collection by numeric id or exact name (manual collections only).
pub async fn resolve_manual_collection_id(
    client: &RommClient,
    query: Option<&str>,
) -> Result<Option<u64>> {
    let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if let Ok(id) = q.parse::<u64>() {
        return Ok(Some(id));
    }
    let list = client.call(&ListCollections).await?.into_vec();
    let matches = match_collections_by_name(q, &list);
    match matches.len() {
        0 => Err(anyhow!(
            "No manual collection named '{}'. Use `romm-cli collections list`.",
            q
        )),
        1 => Ok(Some(matches[0].id)),
        _ => Err(anyhow!(
            "Manual collection '{}' is ambiguous among {} matches; use a numeric id.",
            q,
            matches.len()
        )),
    }
}

/// Resolve smart collection by numeric id or exact name.
pub async fn resolve_smart_collection_id(
    client: &RommClient,
    query: Option<&str>,
) -> Result<Option<u64>> {
    let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if let Ok(id) = q.parse::<u64>() {
        return Ok(Some(id));
    }
    let list = client.call(&ListSmartCollections).await?.into_vec();
    let matches = match_collections_by_name(q, &list);
    match matches.len() {
        0 => Err(anyhow!(
            "No smart collection named '{}'. Use `romm-cli collections list`.",
            q
        )),
        1 => Ok(Some(matches[0].id)),
        _ => Err(anyhow!(
            "Smart collection '{}' is ambiguous among {} matches; use a numeric id.",
            q,
            matches.len()
        )),
    }
}
