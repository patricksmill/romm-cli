//! Shared name/slug → ID resolution for platforms and collections.

use crate::client::RommClient;
use crate::endpoints::collections::{ListCollections, ListSmartCollections};
use crate::endpoints::platforms::ListPlatforms;
use crate::error::RommError;
use crate::types::{Collection, Platform};

/// Resolves a platform ID from a string query by matching against slugs, names, and custom names.
pub fn resolve_platform_id_from_list(
    query: &str,
    platforms: &[Platform],
) -> Result<u64, RommError> {
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
        0 => Err(RommError::Other(format!(
            "No platform found for '{query}'. Use 'romm-cli platforms list' to inspect available values."
        ))),
        _ => {
            let names = exact_name_matches
                .iter()
                .map(|p| format!("{} ({})", p.name, p.id))
                .collect::<Vec<_>>()
                .join(", ");
            Err(RommError::Other(format!(
                "Platform '{query}' is ambiguous. Matches: {names}. Please use a more specific --platform value."
            )))
        }
    }
}

/// Resolves a platform query (slug or name) to a numeric ID.
pub async fn resolve_platform_id(
    client: &RommClient,
    platform_query: Option<&str>,
) -> Result<Option<u64>, RommError> {
    let Some(query) = platform_query.map(str::trim).filter(|q| !q.is_empty()) else {
        return Ok(None);
    };
    let platforms = client.call(&ListPlatforms).await?;
    resolve_platform_id_from_list(query, &platforms).map(Some)
}

/// Resolves multiple platform queries to a list of unique numeric IDs.
pub async fn resolve_platform_ids(
    client: &RommClient,
    names: &[String],
) -> Result<Vec<u64>, RommError> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let platforms = client.call(&ListPlatforms).await?;
    let mut out = Vec::new();
    for n in names {
        let id = resolve_platform_id_from_list(n.trim(), &platforms)?;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    Ok(out)
}

fn match_collections_by_name<'a>(q: &str, collections: &'a [Collection]) -> Vec<&'a Collection> {
    let n = q.trim().to_ascii_lowercase();
    collections
        .iter()
        .filter(|c| c.name.eq_ignore_ascii_case(&n))
        .collect()
}

/// Resolves a manual collection by ID or exact name.
pub async fn resolve_manual_collection_id(
    client: &RommClient,
    query: Option<&str>,
) -> Result<Option<u64>, RommError> {
    let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if let Ok(id) = q.parse::<u64>() {
        return Ok(Some(id));
    }
    let list = client.call(&ListCollections).await?.into_vec();
    let matches = match_collections_by_name(q, &list);
    match matches.len() {
        0 => Err(RommError::Other(format!(
            "No manual collection named '{q}'. Use `romm-cli collections list`."
        ))),
        1 => Ok(Some(matches[0].id)),
        _ => Err(RommError::Other(format!(
            "Manual collection '{q}' is ambiguous among {} matches; use a numeric id.",
            matches.len()
        ))),
    }
}

/// Resolves a smart collection by ID or exact name.
pub async fn resolve_smart_collection_id(
    client: &RommClient,
    query: Option<&str>,
) -> Result<Option<u64>, RommError> {
    let Some(q) = query.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if let Ok(id) = q.parse::<u64>() {
        return Ok(Some(id));
    }
    let list = client.call(&ListSmartCollections).await?.into_vec();
    let matches = match_collections_by_name(q, &list);
    match matches.len() {
        0 => Err(RommError::Other(format!(
            "No smart collection named '{q}'. Use `romm-cli collections list`."
        ))),
        1 => Ok(Some(matches[0].id)),
        _ => Err(RommError::Other(format!(
            "Smart collection '{q}' is ambiguous among {} matches; use a numeric id.",
            matches.len()
        ))),
    }
}
