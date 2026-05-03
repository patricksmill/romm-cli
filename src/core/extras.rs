//! Build download targets for ROM extras (related archives, cover, manual).
//!
//! Shared by the CLI `download extras` subcommand and the TUI extras picker.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::client::RommClient;
use crate::core::utils;
use crate::endpoints::roms::GetRoms;
use crate::services::RomService;
use crate::types::Rom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadAssetKind {
    RomArchive,
    Cover,
    Manual,
}

impl DownloadAssetKind {
    pub fn folder_name(self) -> &'static str {
        match self {
            DownloadAssetKind::RomArchive => "roms",
            DownloadAssetKind::Cover => "covers",
            DownloadAssetKind::Manual => "manuals",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DownloadAssetKind::RomArchive => "ROM archive",
            DownloadAssetKind::Cover => "cover",
            DownloadAssetKind::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTarget {
    pub kind: DownloadAssetKind,
    pub title: String,
    pub source_url: String,
    pub source_query: Vec<(String, String)>,
    pub destination: PathBuf,
}

/// Full extras set for one ROM (API fetch + discovery).
pub async fn build_extras_targets(
    client: &RommClient,
    rom_id: u64,
    output_dir: &Path,
) -> Result<Vec<DownloadTarget>> {
    let service = RomService::new(client);
    let rom = service.get_rom(rom_id).await?;
    let extras_root = extras_root_dir(output_dir, &rom);

    let mut targets = Vec::new();
    targets.extend(build_related_rom_targets(client, &rom, &extras_root).await?);
    if let Some(cover) = build_cover_target(&rom, &extras_root) {
        targets.push(cover);
    }
    if let Some(manual) = build_manual_target(&rom, &extras_root) {
        targets.push(manual);
    }

    Ok(targets)
}

async fn build_related_rom_targets(
    client: &RommClient,
    rom: &Rom,
    extras_root: &Path,
) -> Result<Vec<DownloadTarget>> {
    let service = RomService::new(client);
    let ep = GetRoms {
        search_term: Some(rom.name.clone()),
        platform_id: Some(rom.platform_id),
        limit: Some(9999),
        ..Default::default()
    };
    let results = service.search_roms(&ep).await?;
    let groups = utils::group_roms_by_name(&results.items);
    let Some(group) = groups.iter().find(|g| g.name == rom.name) else {
        return Ok(Vec::new());
    };

    let mut targets = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut push_rom = |candidate: &Rom| {
        if candidate.id == rom.id || !seen.insert(candidate.id) {
            return;
        }
        targets.push(related_rom_download_target(rom, candidate, extras_root));
    };

    push_rom(&group.primary);
    for other in &group.others {
        push_rom(other);
    }
    Ok(targets)
}

/// One related ROM archive under `extras_root` (same layout as CLI extras).
pub fn related_rom_download_target(_parent: &Rom, candidate: &Rom, extras_root: &Path) -> DownloadTarget {
    let name = sanitize_extra_file_name(&candidate.fs_name);
    DownloadTarget {
        kind: DownloadAssetKind::RomArchive,
        title: candidate.fs_name.clone(),
        source_url: "/api/roms/download".to_string(),
        source_query: vec![
            ("rom_ids".into(), candidate.id.to_string()),
            ("filename".into(), name.clone()),
        ],
        destination: extras_root
            .join(DownloadAssetKind::RomArchive.folder_name())
            .join(name),
    }
}

pub fn build_cover_target(rom: &Rom, extras_root: &Path) -> Option<DownloadTarget> {
    let url = rom
        .url_cover
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())?;
    let filename = filename_from_url(url, "cover");
    Some(DownloadTarget {
        kind: DownloadAssetKind::Cover,
        title: rom.name.clone(),
        source_url: url.to_string(),
        source_query: Vec::new(),
        destination: extras_root
            .join(DownloadAssetKind::Cover.folder_name())
            .join(filename),
    })
}

pub fn build_manual_target(rom: &Rom, extras_root: &Path) -> Option<DownloadTarget> {
    let url = rom
        .url_manual
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())?;
    let filename = filename_from_url(url, "manual");
    Some(DownloadTarget {
        kind: DownloadAssetKind::Manual,
        title: rom.name.clone(),
        source_url: url.to_string(),
        source_query: Vec::new(),
        destination: extras_root
            .join(DownloadAssetKind::Manual.folder_name())
            .join(filename),
    })
}

pub fn extras_root_dir(output_dir: &Path, rom: &Rom) -> PathBuf {
    let platform_slug = rom
        .platform_fs_slug
        .clone()
        .or_else(|| rom.platform_slug.clone())
        .unwrap_or_else(|| format!("platform-{}", rom.platform_id));
    let game_slug = sanitized_extra_game_name(&rom.name, rom.id);
    output_dir
        .join(utils::sanitize_filename(&platform_slug))
        .join(game_slug)
        .join("extras")
}

fn sanitized_extra_game_name(name: &str, rom_id: u64) -> String {
    let sanitized = utils::sanitize_filename(name);
    if sanitized.trim().is_empty() {
        format!("rom-{rom_id}")
    } else {
        sanitized
    }
}

fn sanitize_extra_file_name(name: &str) -> String {
    let sanitized = utils::sanitize_filename(name);
    if sanitized.trim().is_empty() {
        "download.bin".to_string()
    } else {
        sanitized
    }
}

fn filename_from_url(url: &str, fallback: &str) -> String {
    let fallback = sanitize_extra_file_name(fallback);
    reqwest::Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next_back().map(str::to_string))
        })
        .map(|name| sanitize_extra_file_name(&name))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Rom;

    #[test]
    fn extras_root_dir_is_sanitized() {
        let rom = rom_fixture(7, "Mario Kart", "Mario Kart [USA].zip");
        let dir = extras_root_dir(PathBuf::from("/tmp/out").as_path(), &rom);
        assert_eq!(
            dir,
            PathBuf::from("/tmp/out")
                .join("Nintendo Switch")
                .join("Mario Kart")
                .join("extras")
        );
    }

    #[test]
    fn filename_from_url_uses_remote_leaf_or_fallback() {
        assert_eq!(
            filename_from_url("https://example.com/files/guide.pdf?download=1", "manual"),
            "guide.pdf"
        );
        assert_eq!(filename_from_url("not-a-url", "manual"), "manual");
    }

    #[test]
    fn build_cover_and_manual_when_urls_present() {
        let mut rom = rom_fixture(1, "Game", "game.zip");
        rom.url_cover = Some("https://cdn.example.com/cover.png".into());
        rom.url_manual = Some("https://cdn.example.com/doc.pdf".into());
        let root = PathBuf::from("/out/extras");
        let cover = build_cover_target(&rom, &root).expect("cover");
        assert_eq!(cover.kind, DownloadAssetKind::Cover);
        assert!(cover.destination.ends_with("covers/cover.png"));
        let manual = build_manual_target(&rom, &root).expect("manual");
        assert_eq!(manual.kind, DownloadAssetKind::Manual);
        assert!(manual.destination.ends_with("manuals/doc.pdf"));
    }

    #[test]
    fn build_cover_skips_when_missing_url() {
        let rom = rom_fixture(1, "Game", "game.zip");
        let root = PathBuf::from("/out/extras");
        assert!(build_cover_target(&rom, &root).is_none());
        assert!(build_manual_target(&rom, &root).is_none());
    }

    fn rom_fixture(id: u64, name: &str, fs_name: &str) -> Rom {
        Rom {
            id,
            platform_id: 1,
            platform_slug: Some("switch".to_string()),
            platform_fs_slug: Some("Nintendo Switch".to_string()),
            platform_custom_name: None,
            platform_display_name: None,
            fs_name: fs_name.to_string(),
            fs_name_no_tags: name.to_string(),
            fs_name_no_ext: name.to_string(),
            fs_extension: "zip".to_string(),
            fs_path: format!("/{id}.zip"),
            fs_size_bytes: 1,
            name: name.to_string(),
            slug: None,
            summary: None,
            path_cover_small: None,
            path_cover_large: None,
            url_cover: None,
            has_manual: false,
            path_manual: None,
            url_manual: None,
            is_unidentified: false,
            is_identified: true,
        }
    }
}
