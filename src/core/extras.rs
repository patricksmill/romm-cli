//! Build download targets for ROM extras (related archives, cover, manual).
//!
//! Shared by the CLI `download extras` subcommand and the TUI extras picker.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::client::RommClient;
use crate::config::RomsLayoutConfig;
use crate::core::download::resolve_console_roms_dir;
use crate::core::utils;
use crate::endpoints::roms::GetRoms;
use crate::endpoints::roms::GetRom;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::types::{Rom, RomFile, RomFileCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadAssetKind {
    RomArchive,
    RomFile,
    Cover,
    Manual,
}

impl DownloadAssetKind {
    pub fn folder_name(self) -> &'static str {
        match self {
            DownloadAssetKind::RomArchive => "roms",
            DownloadAssetKind::RomFile => "roms",
            DownloadAssetKind::Cover => "covers",
            DownloadAssetKind::Manual => "manuals",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DownloadAssetKind::RomArchive => "ROM archive",
            DownloadAssetKind::RomFile => "ROM file",
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
    pub expected_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalRomFileGroup {
    BaseGame,
    Update,
    Dlc,
}

/// Full extras set for one ROM (API fetch + discovery).
pub async fn build_extras_targets(
    client: &RommClient,
    rom_id: u64,
    layout: &RomsLayoutConfig,
    base_dir: &Path,
) -> Result<Vec<DownloadTarget>> {
    let rom = client.call(&GetRom { id: rom_id }).await?;
    let extras_root = extras_root_dir(layout, base_dir, &rom)?;

    let mut targets = Vec::new();
    targets.extend(build_internal_extra_targets(&rom, layout, base_dir)?);
    targets.extend(build_related_rom_targets(client, &rom, &extras_root).await?);
    if let Some(cover) = build_cover_target(&rom, &extras_root) {
        targets.push(cover);
    }
    if let Some(manual) = build_manual_target(&rom, &extras_root) {
        targets.push(manual);
    }

    Ok(targets)
}

/// Updates/DLC set for the normal single-ROM download follow-up prompt.
pub async fn build_update_dlc_targets_for_rom(
    client: &RommClient,
    rom: &Rom,
    layout: &RomsLayoutConfig,
    base_dir: &Path,
) -> Result<Vec<DownloadTarget>> {
    let extras_root = extras_root_dir(layout, base_dir, rom)?;
    let related_rows = related_rom_rows(client, rom).await?;
    build_update_dlc_targets_from_related_rows(rom, &related_rows, layout, base_dir, &extras_root)
}

pub fn has_update_or_dlc_extras(rom: &Rom, related_rows: &[Rom]) -> bool {
    !internal_file_subset(rom, InternalRomFileGroup::Update).is_empty()
        || !internal_file_subset(rom, InternalRomFileGroup::Dlc).is_empty()
        || !related_rows.is_empty()
}

pub fn build_base_rom_file_targets(
    rom: &Rom,
    layout: &RomsLayoutConfig,
    base_dir: &Path,
) -> Result<Vec<DownloadTarget>> {
    let base_files = internal_file_subset(rom, InternalRomFileGroup::BaseGame);
    if base_files.is_empty() {
        return Ok(Vec::new());
    }
    let platform_dir = resolve_console_roms_dir(layout, base_dir, rom)?;
    Ok(base_files
        .into_iter()
        .map(|file| {
            internal_rom_file_target(rom, file, &platform_dir, InternalRomFileGroup::BaseGame)
        })
        .collect())
}

pub fn build_update_dlc_file_targets_for_rom(
    rom: &Rom,
    layout: &RomsLayoutConfig,
    base_dir: &Path,
) -> Result<Vec<DownloadTarget>> {
    build_internal_extra_targets(rom, layout, base_dir)
}

pub fn collect_update_dlc_files(rom: &Rom) -> Vec<RomFile> {
    let mut out = internal_file_subset(rom, InternalRomFileGroup::Update);
    out.extend(internal_file_subset(rom, InternalRomFileGroup::Dlc));
    out
}

async fn build_related_rom_targets(
    client: &RommClient,
    rom: &Rom,
    extras_root: &Path,
) -> Result<Vec<DownloadTarget>> {
    let related_rows = related_rom_rows(client, rom).await?;
    Ok(related_rows
        .iter()
        .map(|candidate| related_rom_download_target(rom, candidate, extras_root))
        .collect())
}

async fn related_rom_rows(client: &RommClient, rom: &Rom) -> Result<Vec<Rom>> {
    let ep = GetRoms {
        search_term: Some(rom.name.clone()),
        platform_id: Some(rom.platform_id),
        limit: Some(9999),
        ..Default::default()
    };
    let results = client.call(&ep).await?;
    let groups = utils::group_roms_by_name(&results.items);
    let Some(group) = groups.iter().find(|g| g.name == rom.name) else {
        return Ok(Vec::new());
    };

    let mut rows = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut push_rom = |candidate: &Rom| {
        if candidate.id == rom.id || !seen.insert(candidate.id) {
            return;
        }
        rows.push(candidate.clone());
    };

    push_rom(&group.primary);
    for other in &group.others {
        push_rom(other);
    }
    Ok(rows)
}

pub fn build_update_dlc_targets_from_related_rows(
    rom: &Rom,
    related_rows: &[Rom],
    layout: &RomsLayoutConfig,
    base_dir: &Path,
    extras_root: &Path,
) -> Result<Vec<DownloadTarget>> {
    let mut targets = build_internal_extra_targets(rom, layout, base_dir)?;
    targets.extend(
        related_rows
            .iter()
            .map(|candidate| related_rom_download_target(rom, candidate, extras_root)),
    );
    Ok(targets)
}

fn build_internal_extra_targets(
    rom: &Rom,
    layout: &RomsLayoutConfig,
    base_dir: &Path,
) -> Result<Vec<DownloadTarget>> {
    let updates = internal_file_subset(rom, InternalRomFileGroup::Update);
    let dlc = internal_file_subset(rom, InternalRomFileGroup::Dlc);
    let mut out = Vec::with_capacity(updates.len() + dlc.len());
    let platform_dir = resolve_console_roms_dir(layout, base_dir, rom)?;
    let game_dir = sanitized_extra_game_name(&rom.name, rom.id);
    for f in updates {
        out.push(internal_rom_file_target(
            rom,
            f,
            &platform_dir.join("updates").join(&game_dir),
            InternalRomFileGroup::Update,
        ));
    }
    for f in dlc {
        out.push(internal_rom_file_target(
            rom,
            f,
            &platform_dir.join("dlc").join(&game_dir),
            InternalRomFileGroup::Dlc,
        ));
    }
    Ok(out)
}

/// One related ROM archive under `extras_root` (same layout as CLI extras).
pub fn related_rom_download_target(
    _parent: &Rom,
    candidate: &Rom,
    extras_root: &Path,
) -> DownloadTarget {
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
        expected_size_bytes: None,
    }
}

fn internal_rom_file_target(
    _parent: &Rom,
    file: RomFile,
    destination_dir: &Path,
    group: InternalRomFileGroup,
) -> DownloadTarget {
    let encoded_name = utf8_percent_encode(&file.file_name, NON_ALPHANUMERIC).to_string();
    let source_url = format!("/api/roms/{}/files/content/{}", file.id, encoded_name);
    let title = match group {
        InternalRomFileGroup::BaseGame => file.file_name.clone(),
        InternalRomFileGroup::Update => format!("Update: {}", file.file_name),
        InternalRomFileGroup::Dlc => format!("DLC: {}", file.file_name),
    };
    let output_name = sanitize_extra_file_name(&file.file_name);
    DownloadTarget {
        kind: DownloadAssetKind::RomFile,
        title,
        source_url,
        source_query: Vec::new(),
        destination: destination_dir.join(output_name),
        expected_size_bytes: Some(file.file_size_bytes),
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
        expected_size_bytes: None,
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
        expected_size_bytes: None,
    })
}

pub fn extras_root_dir(layout: &RomsLayoutConfig, base_dir: &Path, rom: &Rom) -> Result<PathBuf> {
    let platform_dir = resolve_console_roms_dir(layout, base_dir, rom)?;
    let game_slug = sanitized_extra_game_name(&rom.name, rom.id);
    Ok(platform_dir.join(game_slug).join("extras"))
}

fn internal_file_subset(rom: &Rom, group: InternalRomFileGroup) -> Vec<RomFile> {
    rom.files
        .iter()
        .filter(|f| match group {
            InternalRomFileGroup::BaseGame => is_base_game_file(f),
            InternalRomFileGroup::Update => is_update_file(f),
            InternalRomFileGroup::Dlc => is_dlc_file(f),
        })
        .cloned()
        .collect()
}

fn is_update_file(file: &RomFile) -> bool {
    if matches!(file.category, Some(RomFileCategory::Update)) {
        return true;
    }
    filename_has_token(&file.file_name, &["update", "upd"])
}

fn is_dlc_file(file: &RomFile) -> bool {
    if matches!(file.category, Some(RomFileCategory::Dlc)) {
        return true;
    }
    filename_has_token(&file.file_name, &["dlc", "expansion"])
}

fn is_base_game_file(file: &RomFile) -> bool {
    if matches!(file.category, Some(RomFileCategory::Game)) {
        return true;
    }
    if file.category.is_some() {
        return false;
    }
    !is_update_file(file) && !is_dlc_file(file)
}

fn filename_has_token(name: &str, tokens: &[&str]) -> bool {
    let normalized = name.to_ascii_lowercase();
    let normalized = normalized.replace(['[', ']', '(', ')', '{', '}', '-', '_', '.'], " ");
    normalized
        .split_whitespace()
        .any(|part| tokens.contains(&part))
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
    use crate::config::RomsLayoutConfig;
    use crate::types::Rom;

    fn default_layout() -> RomsLayoutConfig {
        RomsLayoutConfig::default()
    }

    #[test]
    fn extras_root_dir_is_sanitized() {
        let rom = rom_fixture(7, "Mario Kart", "Mario Kart [USA].zip");
        let dir = extras_root_dir(&default_layout(), Path::new("/tmp/out"), &rom).unwrap();
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
            files: Vec::new(),
        }
    }

    #[test]
    fn base_file_targets_build_from_internal_game_files() {
        let mut rom = rom_fixture(1, "Game", "pack.zip");
        rom.files = vec![
            RomFile {
                id: 10,
                rom_id: 1,
                file_name: "base.nsp".into(),
                file_path: "/base.nsp".into(),
                file_size_bytes: 10,
                category: Some(RomFileCategory::Game),
            },
            RomFile {
                id: 11,
                rom_id: 1,
                file_name: "upd.nsp".into(),
                file_path: "/upd.nsp".into(),
                file_size_bytes: 10,
                category: Some(RomFileCategory::Update),
            },
        ];
        let targets =
            build_base_rom_file_targets(&rom, &default_layout(), Path::new("/tmp/out")).unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].kind, DownloadAssetKind::RomFile);
        assert_eq!(
            targets[0].source_url,
            "/api/roms/10/files/content/base%2Ensp"
        );
        assert!(targets[0].destination.ends_with("Nintendo Switch/base.nsp"));
    }

    #[test]
    fn detects_update_dlc_by_filename_when_category_missing() {
        let mut rom = rom_fixture(1, "Game", "pack.zip");
        rom.files = vec![
            RomFile {
                id: 10,
                rom_id: 1,
                file_name: "Game [v1.4.0] [Update].nsp".into(),
                file_path: "/upd.nsp".into(),
                file_size_bytes: 10,
                category: None,
            },
            RomFile {
                id: 11,
                rom_id: 1,
                file_name: "Game [DLC].nsp".into(),
                file_path: "/dlc.nsp".into(),
                file_size_bytes: 10,
                category: None,
            },
            RomFile {
                id: 12,
                rom_id: 1,
                file_name: "Game Base.nsp".into(),
                file_path: "/base.nsp".into(),
                file_size_bytes: 10,
                category: None,
            },
        ];
        assert!(has_update_or_dlc_extras(&rom, &[]));
        let base =
            build_base_rom_file_targets(&rom, &default_layout(), Path::new("/tmp/out")).unwrap();
        assert_eq!(base.len(), 1);
        assert!(base[0]
            .destination
            .ends_with("Nintendo Switch/Game Base.nsp"));
        let extras =
            build_update_dlc_file_targets_for_rom(&rom, &default_layout(), Path::new("/tmp/out"))
                .unwrap();
        assert_eq!(extras.len(), 2);
        assert_eq!(
            extras[0].source_url,
            "/api/roms/10/files/content/Game%20%5Bv1%2E4%2E0%5D%20%5BUpdate%5D%2Ensp"
        );
        assert!(extras[0]
            .destination
            .ends_with("Nintendo Switch/updates/Game/Game _v1.4.0_ _Update_.nsp"));
        assert_eq!(
            extras[1].source_url,
            "/api/roms/11/files/content/Game%20%5BDLC%5D%2Ensp"
        );
        assert!(extras[1]
            .destination
            .ends_with("Nintendo Switch/dlc/Game/Game _DLC_.nsp"));
    }

    #[test]
    fn update_dlc_targets_include_related_roms_but_not_cover_or_manual() {
        let mut rom = rom_fixture(1, "Game", "pack.zip");
        rom.url_cover = Some("https://example.com/cover.png".into());
        rom.url_manual = Some("https://example.com/manual.pdf".into());
        rom.files = vec![RomFile {
            id: 10,
            rom_id: 1,
            file_name: "Game [Update].nsp".into(),
            file_path: "/upd.nsp".into(),
            file_size_bytes: 10,
            category: Some(RomFileCategory::Update),
        }];
        let related = Rom {
            id: 2,
            fs_name: "Game DLC.zip".into(),
            ..rom_fixture(2, "Game", "Game DLC.zip")
        };
        let extras_root = extras_root_dir(&default_layout(), Path::new("/tmp/out"), &rom).unwrap();

        let targets = build_update_dlc_targets_from_related_rows(
            &rom,
            &[related],
            &default_layout(),
            Path::new("/tmp/out"),
            &extras_root,
        )
        .unwrap();

        assert_eq!(targets.len(), 2);
        assert!(targets.iter().any(|t| t.kind == DownloadAssetKind::RomFile));
        assert!(targets
            .iter()
            .any(|t| t.kind == DownloadAssetKind::RomArchive));
        assert!(!targets.iter().any(|t| t.kind == DownloadAssetKind::Cover));
        assert!(!targets.iter().any(|t| t.kind == DownloadAssetKind::Manual));
    }
}
