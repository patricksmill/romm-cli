use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Row from `GET /api/search/roms`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SearchRom {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    pub platform_id: u64,
    #[serde(default)]
    pub igdb_id: Option<i64>,
    #[serde(default)]
    pub moby_id: Option<i64>,
    #[serde(default)]
    pub ss_id: Option<i64>,
    #[serde(default)]
    pub launchbox_id: Option<i64>,
    #[serde(default)]
    pub flashpoint_id: Option<String>,
    #[serde(default)]
    pub sgdb_id: Option<i64>,
    #[serde(default)]
    pub is_identified: bool,
    #[serde(default)]
    pub is_unidentified: bool,
    #[serde(default)]
    pub igdb_url_cover: Option<String>,
    #[serde(default)]
    pub ss_url_cover: Option<String>,
    #[serde(default)]
    pub moby_url_cover: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Provider IDs to send in `PUT /api/roms/{id}` when applying a search match.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct RomMatchFields {
    pub igdb_id: Option<i64>,
    pub moby_id: Option<i64>,
    pub ss_id: Option<i64>,
    pub launchbox_id: Option<i64>,
    pub flashpoint_id: Option<String>,
    pub sgdb_id: Option<i64>,
    pub ra_id: Option<i64>,
    pub hasheous_id: Option<i64>,
    pub tgdb_id: Option<i64>,
    pub hltb_id: Option<i64>,
    pub libretro_id: Option<String>,
}

impl SearchRom {
    /// Fields to apply when the user picks this search row (non-null IDs only).
    pub fn primary_match_fields(&self) -> RomMatchFields {
        RomMatchFields {
            igdb_id: self.igdb_id,
            moby_id: self.moby_id,
            ss_id: self.ss_id,
            launchbox_id: self.launchbox_id,
            flashpoint_id: self.flashpoint_id.clone(),
            sgdb_id: self.sgdb_id,
            ..Default::default()
        }
    }
}

impl RomMatchFields {
    pub fn is_empty(&self) -> bool {
        self.igdb_id.is_none()
            && self.moby_id.is_none()
            && self.ss_id.is_none()
            && self.launchbox_id.is_none()
            && self.flashpoint_id.is_none()
            && self.sgdb_id.is_none()
            && self.ra_id.is_none()
            && self.hasheous_id.is_none()
            && self.tgdb_id.is_none()
            && self.hltb_id.is_none()
            && self.libretro_id.is_none()
    }
}

/// Row from `GET /api/search/cover` (SteamGridDB).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SearchCover {
    pub name: String,
    #[serde(default)]
    pub resources: Vec<SgdbResource>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SgdbResource {
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Subset of `DetailedRomSchema` returned by `PUT /api/roms/{id}`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RomUpdateResponse {
    pub id: u64,
    pub platform_id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub url_cover: Option<String>,
    #[serde(default)]
    pub path_cover_small: Option<String>,
    #[serde(default)]
    pub path_cover_large: Option<String>,
    #[serde(default)]
    pub is_identified: bool,
    #[serde(default)]
    pub is_unidentified: bool,
    #[serde(flatten)]
    pub extra: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_rom_deserializes_demo_shape() {
        let json = r#"{
            "name": "Super Mario Bros.",
            "slug": "super-mario-bros",
            "summary": "A platformer.",
            "platform_id": 1,
            "igdb_id": 1234,
            "ss_id": null,
            "moby_id": null,
            "is_identified": true,
            "is_unidentified": false,
            "igdb_url_cover": "https://example.com/cover.jpg"
        }"#;
        let row: SearchRom = serde_json::from_str(json).unwrap();
        assert_eq!(row.name, "Super Mario Bros.");
        assert_eq!(row.igdb_id, Some(1234));
        assert_eq!(row.primary_match_fields().igdb_id, Some(1234));
    }
}
