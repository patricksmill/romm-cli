//! ROM-related API endpoints.
//!
//! This module contains endpoint definitions for searching, retrieving,
//! updating, and deleting ROMs, as well as managing ROM metadata and identification.

use crate::types::RomList;

use super::Endpoint;
use serde_json::{json, Value};

fn push_bool(q: &mut Vec<(String, String)>, key: &str, v: Option<bool>) {
    if let Some(b) = v {
        q.push((key.into(), b.to_string()));
    }
}

fn push_str(q: &mut Vec<(String, String)>, key: &str, v: &Option<String>) {
    if let Some(s) = v {
        if !s.is_empty() {
            q.push((key.into(), s.clone()));
        }
    }
}

fn push_str_list(q: &mut Vec<(String, String)>, key: &str, items: &[String]) {
    for it in items {
        q.push((key.into(), it.clone()));
    }
}

/// Retrieve ROMs with optional filters (`GET /api/roms`).
#[derive(Debug, Default, Clone)]
pub struct GetRoms {
    pub search_term: Option<String>,
    /// When set, emits one `platform_ids` query entry.
    pub platform_id: Option<u64>,
    /// Additional platform IDs (repeat `platform_ids` in the query).
    pub platform_ids: Vec<u64>,
    pub collection_id: Option<u64>,
    pub smart_collection_id: Option<u64>,
    pub virtual_collection_id: Option<String>,
    pub matched: Option<bool>,
    pub favorite: Option<bool>,
    pub duplicate: Option<bool>,
    pub last_played: Option<bool>,
    pub playable: Option<bool>,
    pub missing: Option<bool>,
    pub has_ra: Option<bool>,
    pub verified: Option<bool>,
    pub group_by_meta_id: Option<bool>,
    pub genres: Vec<String>,
    pub franchises: Vec<String>,
    pub collections: Vec<String>,
    pub companies: Vec<String>,
    pub age_ratings: Vec<String>,
    pub statuses: Vec<String>,
    pub regions: Vec<String>,
    pub languages: Vec<String>,
    pub player_counts: Vec<String>,
    pub genres_logic: Option<String>,
    pub franchises_logic: Option<String>,
    pub collections_logic: Option<String>,
    pub companies_logic: Option<String>,
    pub age_ratings_logic: Option<String>,
    pub regions_logic: Option<String>,
    pub languages_logic: Option<String>,
    pub statuses_logic: Option<String>,
    pub player_counts_logic: Option<String>,
    pub order_by: Option<String>,
    pub order_dir: Option<String>,
    pub updated_after: Option<String>,
    pub with_char_index: Option<bool>,
    pub with_filter_values: Option<bool>,
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

        let mut seen = std::collections::HashSet::new();
        for pid in &self.platform_ids {
            if seen.insert(*pid) {
                q.push(("platform_ids".into(), pid.to_string()));
            }
        }
        if let Some(pid) = self.platform_id {
            if seen.insert(pid) {
                q.push(("platform_ids".into(), pid.to_string()));
            }
        }

        if let Some(cid) = self.collection_id {
            q.push(("collection_id".into(), cid.to_string()));
        }
        if let Some(sid) = self.smart_collection_id {
            q.push(("smart_collection_id".into(), sid.to_string()));
        }
        if let Some(ref vid) = self.virtual_collection_id {
            q.push(("virtual_collection_id".into(), vid.clone()));
        }

        push_bool(&mut q, "matched", self.matched);
        push_bool(&mut q, "favorite", self.favorite);
        push_bool(&mut q, "duplicate", self.duplicate);
        push_bool(&mut q, "last_played", self.last_played);
        push_bool(&mut q, "playable", self.playable);
        push_bool(&mut q, "missing", self.missing);
        push_bool(&mut q, "has_ra", self.has_ra);
        push_bool(&mut q, "verified", self.verified);
        push_bool(&mut q, "group_by_meta_id", self.group_by_meta_id);
        push_bool(&mut q, "with_char_index", self.with_char_index);
        push_bool(&mut q, "with_filter_values", self.with_filter_values);

        push_str_list(&mut q, "genres", &self.genres);
        push_str_list(&mut q, "franchises", &self.franchises);
        push_str_list(&mut q, "collections", &self.collections);
        push_str_list(&mut q, "companies", &self.companies);
        push_str_list(&mut q, "age_ratings", &self.age_ratings);
        push_str_list(&mut q, "statuses", &self.statuses);
        push_str_list(&mut q, "regions", &self.regions);
        push_str_list(&mut q, "languages", &self.languages);
        push_str_list(&mut q, "player_counts", &self.player_counts);

        push_str(&mut q, "genres_logic", &self.genres_logic);
        push_str(&mut q, "franchises_logic", &self.franchises_logic);
        push_str(&mut q, "collections_logic", &self.collections_logic);
        push_str(&mut q, "companies_logic", &self.companies_logic);
        push_str(&mut q, "age_ratings_logic", &self.age_ratings_logic);
        push_str(&mut q, "regions_logic", &self.regions_logic);
        push_str(&mut q, "languages_logic", &self.languages_logic);
        push_str(&mut q, "statuses_logic", &self.statuses_logic);
        push_str(&mut q, "player_counts_logic", &self.player_counts_logic);

        push_str(&mut q, "order_by", &self.order_by);
        push_str(&mut q, "order_dir", &self.order_dir);
        push_str(&mut q, "updated_after", &self.updated_after);

        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            q.push(("offset".into(), offset.to_string()));
        }

        q
    }
}

/// Retrieve a single ROM by ID.
#[derive(Debug, Clone)]
pub struct GetRom {
    pub id: u64,
}

impl Endpoint for GetRom {
    type Output = crate::types::Rom;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        format!("/api/roms/{}", self.id)
    }
}

/// `GET /api/roms/by-hash`
#[derive(Debug, Default, Clone)]
pub struct GetRomByHash {
    pub crc_hash: Option<String>,
    pub md5_hash: Option<String>,
    pub sha1_hash: Option<String>,
}

impl Endpoint for GetRomByHash {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/roms/by-hash".into()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        push_str(&mut q, "crc_hash", &self.crc_hash);
        push_str(&mut q, "md5_hash", &self.md5_hash);
        push_str(&mut q, "sha1_hash", &self.sha1_hash);
        q
    }
}

/// `GET /api/roms/by-metadata-provider`
#[derive(Debug, Default, Clone)]
pub struct GetRomByMetadataProvider {
    pub igdb_id: Option<i64>,
    pub moby_id: Option<i64>,
    pub ss_id: Option<i64>,
    pub ra_id: Option<i64>,
    pub launchbox_id: Option<i64>,
    pub hasheous_id: Option<i64>,
    pub tgdb_id: Option<i64>,
    pub flashpoint_id: Option<String>,
    pub hltb_id: Option<i64>,
}

impl Endpoint for GetRomByMetadataProvider {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/roms/by-metadata-provider".into()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(v) = self.igdb_id {
            q.push(("igdb_id".into(), v.to_string()));
        }
        if let Some(v) = self.moby_id {
            q.push(("moby_id".into(), v.to_string()));
        }
        if let Some(v) = self.ss_id {
            q.push(("ss_id".into(), v.to_string()));
        }
        if let Some(v) = self.ra_id {
            q.push(("ra_id".into(), v.to_string()));
        }
        if let Some(v) = self.launchbox_id {
            q.push(("launchbox_id".into(), v.to_string()));
        }
        if let Some(v) = self.hasheous_id {
            q.push(("hasheous_id".into(), v.to_string()));
        }
        if let Some(v) = self.tgdb_id {
            q.push(("tgdb_id".into(), v.to_string()));
        }
        push_str(&mut q, "flashpoint_id", &self.flashpoint_id);
        if let Some(v) = self.hltb_id {
            q.push(("hltb_id".into(), v.to_string()));
        }
        q
    }
}

/// `GET /api/roms/filters`
#[derive(Debug, Default, Clone)]
pub struct GetRomFilters;

impl Endpoint for GetRomFilters {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/roms/filters".into()
    }
}

/// `POST /api/roms/delete`
#[derive(Debug, Clone)]
pub struct DeleteRoms {
    pub roms: Vec<u64>,
    pub delete_from_fs: Vec<u64>,
}

impl Endpoint for DeleteRoms {
    type Output = Value;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        "/api/roms/delete".into()
    }

    fn body(&self) -> Option<Value> {
        Some(json!({
            "roms": self.roms,
            "delete_from_fs": self.delete_from_fs,
        }))
    }
}

/// `PUT /api/roms/{id}/props` — RomM accepts JSON body plus optional query flags.
#[derive(Debug, Clone)]
pub struct PutRomUserProps {
    pub rom_id: u64,
    pub body: Value,
    pub update_last_played: bool,
    pub remove_last_played: bool,
}

impl Endpoint for PutRomUserProps {
    type Output = Value;

    fn method(&self) -> &'static str {
        "PUT"
    }

    fn path(&self) -> String {
        format!("/api/roms/{}/props", self.rom_id)
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if self.update_last_played {
            q.push(("update_last_played".into(), "true".into()));
        }
        if self.remove_last_played {
            q.push(("remove_last_played".into(), "true".into()));
        }
        q
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `GET /api/roms/{id}/notes`
#[derive(Debug, Clone)]
pub struct GetRomNotes {
    pub rom_id: u64,
    pub public_only: Option<bool>,
    pub search: Option<String>,
    pub tags: Vec<String>,
}

impl Endpoint for GetRomNotes {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        format!("/api/roms/{}/notes", self.rom_id)
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        push_bool(&mut q, "public_only", self.public_only);
        push_str(&mut q, "search", &self.search);
        push_str_list(&mut q, "tags", &self.tags);
        q
    }
}

/// `POST /api/roms/{id}/notes`
#[derive(Debug, Clone)]
pub struct PostRomNote {
    pub rom_id: u64,
    pub body: Value,
}

impl Endpoint for PostRomNote {
    type Output = Value;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        format!("/api/roms/{}/notes", self.rom_id)
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `PUT /api/roms/{id}/notes/{note_id}`
#[derive(Debug, Clone)]
pub struct PutRomNote {
    pub rom_id: u64,
    pub note_id: u64,
    pub body: Value,
}

impl Endpoint for PutRomNote {
    type Output = Value;

    fn method(&self) -> &'static str {
        "PUT"
    }

    fn path(&self) -> String {
        format!("/api/roms/{}/notes/{}", self.rom_id, self.note_id)
    }

    fn body(&self) -> Option<Value> {
        Some(self.body.clone())
    }
}

/// `DELETE /api/roms/{id}/notes/{note_id}`
#[derive(Debug, Clone)]
pub struct DeleteRomNote {
    pub rom_id: u64,
    pub note_id: u64,
}

impl Endpoint for DeleteRomNote {
    type Output = Value;

    fn method(&self) -> &'static str {
        "DELETE"
    }

    fn path(&self) -> String {
        format!("/api/roms/{}/notes/{}", self.rom_id, self.note_id)
    }
}

/// `GET /api/search/cover`
#[derive(Debug, Clone)]
pub struct GetSearchCover {
    pub search_term: String,
}

impl Endpoint for GetSearchCover {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/search/cover".into()
    }

    fn query(&self) -> Vec<(String, String)> {
        vec![("search_term".into(), self.search_term.clone())]
    }
}

/// `GET /api/search/roms`
#[derive(Debug, Clone)]
pub struct GetSearchRoms {
    pub rom_id: u64,
    pub search_term: Option<String>,
    pub search_by: Option<String>,
}

impl Endpoint for GetSearchRoms {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/search/roms".into()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = vec![("rom_id".into(), self.rom_id.to_string())];
        push_str(&mut q, "search_term", &self.search_term);
        push_str(&mut q, "search_by", &self.search_by);
        q
    }
}

#[cfg(test)]
mod tests {
    use super::{Endpoint, GetRoms};

    #[test]
    fn get_roms_query_sends_collection_id() {
        let ep = GetRoms {
            collection_id: Some(7),
            limit: Some(100),
            ..Default::default()
        };
        let q = ep.query();
        assert!(q.iter().any(|(k, v)| k == "collection_id" && v == "7"));
        assert!(!q.iter().any(|(k, _)| k == "smart_collection_id"));
    }

    #[test]
    fn get_roms_query_sends_smart_collection_id() {
        let ep = GetRoms {
            smart_collection_id: Some(3),
            limit: Some(50),
            ..Default::default()
        };
        let q = ep.query();
        assert!(q
            .iter()
            .any(|(k, v)| k == "smart_collection_id" && v == "3"));
        assert!(!q.iter().any(|(k, _)| k == "collection_id"));
        assert!(!q.iter().any(|(k, _)| k == "virtual_collection_id"));
    }

    #[test]
    fn get_roms_query_sends_virtual_collection_id() {
        let ep = GetRoms {
            virtual_collection_id: Some("recent".into()),
            limit: Some(10),
            ..Default::default()
        };
        let q = ep.query();
        assert!(q
            .iter()
            .any(|(k, v)| k == "virtual_collection_id" && v == "recent"));
        assert!(!q.iter().any(|(k, _)| k == "collection_id"));
    }

    #[test]
    fn get_roms_dedupes_platform_ids_with_platform_id() {
        let ep = GetRoms {
            platform_id: Some(5),
            platform_ids: vec![5, 7],
            ..Default::default()
        };
        let q = ep.query();
        let ids: Vec<_> = q
            .iter()
            .filter(|(k, _)| k == "platform_ids")
            .map(|(_, v)| v.as_str())
            .collect();
        assert_eq!(ids, vec!["5", "7"]);
    }
}
