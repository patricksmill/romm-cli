use serde::{Deserialize, Serialize};

/// Represents a firmware file associated with a platform.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Firmware {
    /// Unique identifier for the firmware.
    pub id: u64,
    /// Original file name of the firmware.
    pub file_name: String,
    /// File name without RomM tags.
    pub file_name_no_tags: String,
    /// File name without extension.
    pub file_name_no_ext: String,
    /// File extension (e.g., ".bin").
    pub file_extension: String,
    /// Relative file path within the RomM storage.
    pub file_path: String,
    /// File size in bytes.
    pub file_size_bytes: u64,
    /// Full absolute path to the file.
    pub full_path: String,
    /// Whether the firmware hash has been verified against a database.
    pub is_verified: bool,
    /// CRC32 hash of the file.
    pub crc_hash: String,
    /// MD5 hash of the file.
    pub md5_hash: String,
    /// SHA1 hash of the file.
    pub sha1_hash: String,
    /// True if the file is missing from the filesystem.
    pub missing_from_fs: bool,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 update timestamp.
    pub updated_at: String,
}

/// A gaming platform (console or system) supported by RomM.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Platform {
    /// Unique identifier for the platform.
    pub id: u64,
    /// URL-friendly slug (e.g., "nes").
    pub slug: String,
    /// Filesystem-friendly slug used for directory naming.
    pub fs_slug: String,
    /// Total number of ROMs assigned to this platform.
    pub rom_count: u64,
    /// Canonical name of the platform.
    pub name: String,
    /// IGDB slug for metadata lookup.
    pub igdb_slug: Option<String>,
    /// MobyGames slug for metadata lookup.
    pub moby_slug: Option<String>,
    /// HowLongToBeat slug for metadata lookup.
    pub hltb_slug: Option<String>,
    /// Custom user-defined name for the platform.
    pub custom_name: Option<String>,
    /// IGDB ID for metadata lookup.
    pub igdb_id: Option<i64>,
    /// ScreenScraper ID for metadata lookup.
    pub sgdb_id: Option<i64>,
    /// MobyGames ID for metadata lookup.
    pub moby_id: Option<i64>,
    /// LaunchBox ID for metadata lookup.
    pub launchbox_id: Option<i64>,
    /// ScreenScraper ID for metadata lookup.
    pub ss_id: Option<i64>,
    /// RetroAchievements ID for metadata lookup.
    pub ra_id: Option<i64>,
    /// Hasheous ID for metadata lookup.
    pub hasheous_id: Option<i64>,
    /// The Games DB ID for metadata lookup.
    pub tgdb_id: Option<i64>,
    /// Flashpoint ID for metadata lookup.
    pub flashpoint_id: Option<i64>,
    /// Category of the platform (e.g., "Console", "Handheld").
    pub category: Option<String>,
    /// Console generation (e.g., 3).
    pub generation: Option<i64>,
    /// Name of the platform family (e.g., "Nintendo").
    pub family_name: Option<String>,
    /// Slug of the platform family (e.g., "nintendo").
    pub family_slug: Option<String>,
    /// Official website URL.
    pub url: Option<String>,
    /// URL to the platform logo image.
    pub url_logo: Option<String>,
    /// List of firmware files required or associated with this platform.
    pub firmware: Vec<Firmware>,
    /// Preferred aspect ratio for the platform.
    pub aspect_ratio: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 update timestamp.
    pub updated_at: String,
    /// Total size of all ROMs for this platform in bytes.
    pub fs_size_bytes: u64,
    /// True if the platform is not yet fully identified in the RomM database.
    pub is_unidentified: bool,
    /// True if the platform has been identified and linked to metadata.
    pub is_identified: bool,
    /// True if the platform directory is missing from the filesystem.
    pub missing_from_fs: bool,
    /// Name used for display in the UI (custom name or original name).
    pub display_name: Option<String>,
}

/// Represents a single ROM file and its associated metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rom {
    /// Unique identifier for the ROM.
    pub id: u64,
    /// ID of the parent platform.
    pub platform_id: u64,
    /// Slug of the parent platform.
    pub platform_slug: Option<String>,
    /// Filesystem slug of the parent platform.
    pub platform_fs_slug: Option<String>,
    /// Custom name of the parent platform.
    pub platform_custom_name: Option<String>,
    /// Display name of the parent platform.
    pub platform_display_name: Option<String>,
    /// Name of the ROM file on disk.
    pub fs_name: String,
    /// ROM file name without RomM tags.
    pub fs_name_no_tags: String,
    /// ROM file name without extension.
    pub fs_name_no_ext: String,
    /// File extension of the ROM (e.g., ".nes").
    pub fs_extension: String,
    /// Relative path to the ROM file.
    pub fs_path: String,
    /// Size of the ROM file in bytes.
    pub fs_size_bytes: u64,
    /// Canonical name of the game.
    pub name: String,
    /// URL-friendly slug for the game.
    pub slug: Option<String>,
    /// Brief description or summary of the game.
    pub summary: Option<String>,
    /// Path to a small thumbnail cover image.
    pub path_cover_small: Option<String>,
    /// Path to a large cover image.
    pub path_cover_large: Option<String>,
    /// Original URL of the cover image.
    pub url_cover: Option<String>,
    /// True if the ROM is not yet fully identified.
    pub is_unidentified: bool,
    /// True if the ROM has been identified and linked to metadata.
    pub is_identified: bool,
}

/// A paginated list of ROMs returned by the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RomList {
    /// The list of ROM items in this page.
    pub items: Vec<Rom>,
    /// Total number of ROMs matching the query across all pages.
    pub total: u64,
    /// Maximum number of items returned in this request.
    pub limit: u64,
    /// Number of items skipped from the beginning.
    pub offset: u64,
}

/// Response row from [`GET /api/collections/virtual`](crate::endpoints::collections::ListVirtualCollections).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VirtualCollectionRow {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub collection_type: String,
    #[serde(default)]
    pub rom_count: u64,
    #[serde(default)]
    pub is_virtual: bool,
}

impl From<VirtualCollectionRow> for Collection {
    fn from(v: VirtualCollectionRow) -> Self {
        Self {
            id: 0,
            name: v.name,
            collection_type: Some(v.collection_type),
            rom_count: Some(v.rom_count),
            is_smart: false,
            is_virtual: true,
            virtual_id: Some(v.id),
        }
    }
}

/// Manual / smart / virtual row for the library collections pane.
///
/// Virtual (autogenerated) collections use string IDs from RomM; see [`virtual_id`](Self::virtual_id)
/// and [`is_virtual`](Self::is_virtual). Numeric [`id`](Self::id) is unused (0) for virtual rows.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Collection {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub collection_type: Option<String>,
    pub rom_count: Option<u64>,
    /// Smart collections are listed separately by RomM; used for ROM filter/cache keys.
    #[serde(default)]
    pub is_smart: bool,
    /// Autogenerated / virtual collections from `GET /api/collections/virtual`.
    #[serde(default)]
    pub is_virtual: bool,
    #[serde(default)]
    pub virtual_id: Option<String>,
}
