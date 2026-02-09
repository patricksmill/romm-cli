use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Firmware {
    pub id: u64,
    pub file_name: String,
    pub file_name_no_tags: String,
    pub file_name_no_ext: String,
    pub file_extension: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub full_path: String,
    pub is_verified: bool,
    pub crc_hash: String,
    pub md5_hash: String,
    pub sha1_hash: String,
    pub missing_from_fs: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Platform {
    pub id: u64,
    pub slug: String,
    pub fs_slug: String,
    pub rom_count: u64,
    pub name: String,
    pub igdb_slug: Option<String>,
    pub moby_slug: Option<String>,
    pub hltb_slug: Option<String>,
    pub custom_name: Option<String>,
    pub igdb_id: Option<i64>,
    pub sgdb_id: Option<i64>,
    pub moby_id: Option<i64>,
    pub launchbox_id: Option<i64>,
    pub ss_id: Option<i64>,
    pub ra_id: Option<i64>,
    pub hasheous_id: Option<i64>,
    pub tgdb_id: Option<i64>,
    pub flashpoint_id: Option<i64>,
    pub category: Option<String>,
    pub generation: Option<i64>,
    pub family_name: Option<String>,
    pub family_slug: Option<String>,
    pub url: Option<String>,
    pub url_logo: Option<String>,
    pub firmware: Vec<Firmware>,
    pub aspect_ratio: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub fs_size_bytes: u64,
    pub is_unidentified: bool,
    pub is_identified: bool,
    pub missing_from_fs: bool,
    pub display_name: Option<String>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rom {
    pub id: u64,
    pub platform_id: u64,
    pub platform_slug: Option<String>,
    pub platform_fs_slug: Option<String>,
    pub platform_custom_name: Option<String>,
    pub platform_display_name: Option<String>,
    pub fs_name: String,
    pub fs_name_no_tags: String,
    pub fs_name_no_ext: String,
    pub fs_extension: String,
    pub fs_path: String,
    pub fs_size_bytes: u64,
    pub name: String,
    pub slug: Option<String>,
    pub summary: Option<String>,
    pub path_cover_small: Option<String>,
    pub path_cover_large: Option<String>,
    pub url_cover: Option<String>,
    pub is_unidentified: bool,
    pub is_identified: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RomList {
    pub items: Vec<Rom>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

/// Collection (smart or virtual) from GET /api/collections.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Collection {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub collection_type: Option<String>,
    pub rom_count: Option<u64>,
}

