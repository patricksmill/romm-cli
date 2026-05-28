use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::{default_theme_id, Config, RomsLayoutConfig, SaveSyncConfig};
use crate::core::download::extras_job::finalize_extras_job_status;
use crate::core::download::paths::resolve_download_directory_from_inputs;
use crate::core::download::transfer::{
    candidate_download_urls, final_download_path_for_rom, finalize_download, FinalizeResult,
};
use crate::core::download::{
    extract_zip_archive, prepare_download_target_destination, resolve_console_roms_dir,
    resolve_console_save_dir, resolve_game_save_dir, unique_zip_path, ExtrasItemResult, ExtrasJob,
    ExtrasJobStatus,
};
use crate::core::extras::{build_base_rom_file_targets, DownloadAssetKind, DownloadTarget};
use crate::types::Rom;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn rom_fixture_with_platform(platform_fs_slug: Option<&str>, fs_name: &str) -> Rom {
    Rom {
        id: 42,
        platform_id: 7,
        platform_slug: Some("nintendo-switch".to_string()),
        platform_fs_slug: platform_fs_slug.map(ToString::to_string),
        platform_custom_name: None,
        platform_display_name: None,
        fs_name: fs_name.to_string(),
        fs_name_no_tags: "game".to_string(),
        fs_name_no_ext: "game".to_string(),
        fs_extension: "zip".to_string(),
        fs_path: "/game.zip".to_string(),
        fs_size_bytes: 1,
        name: "Game".to_string(),
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
fn extras_job_percent_tracks_completed_items() {
    let mut j = ExtrasJob::new(1, "Zelda".into(), "NES".into(), 4);
    assert_eq!(j.percent(), 0);
    j.completed_items = 2;
    assert_eq!(j.percent(), 50);
    j.completed_items = 4;
    assert_eq!(j.percent(), 100);
}

#[test]
fn finalize_extras_job_status_reflects_failures() {
    use crate::core::extras::DownloadAssetKind;

    let ok = ExtrasItemResult {
        title: "a".into(),
        kind: DownloadAssetKind::Cover,
        ok: true,
        error: None,
    };
    let bad = ExtrasItemResult {
        title: "b".into(),
        kind: DownloadAssetKind::Manual,
        ok: false,
        error: Some("e".into()),
    };
    assert_eq!(
        finalize_extras_job_status(&[ok.clone(), ok.clone()]),
        ExtrasJobStatus::Done
    );
    assert_eq!(
        finalize_extras_job_status(&[bad.clone(), bad.clone()]),
        ExtrasJobStatus::AllFailed
    );
    assert_eq!(
        finalize_extras_job_status(&[ok, bad]),
        ExtrasJobStatus::PartialFailure(1)
    );
}

#[test]
fn unique_zip_path_skips_existing_files() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("romm-dl-test-{ts}"));
    std::fs::create_dir_all(&dir).unwrap();
    let p1 = dir.join("game.zip");
    std::fs::File::create(&p1).unwrap().write_all(b"x").unwrap();
    let p2 = unique_zip_path(&dir, "game");
    assert_eq!(p2.file_name().unwrap(), "game__2.zip");
    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn resolve_download_directory_rejects_empty_configured_path() {
    let err = resolve_download_directory_from_inputs(Some("   "), None)
        .expect_err("empty configured path should be rejected");
    assert!(
        err.to_string().contains("cannot be empty"),
        "unexpected error: {err:#}"
    );
}

#[test]
fn resolve_download_directory_creates_missing_nested_directory() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("romm-dl-resolve-{ts}"));
    let nested = base.join("a").join("b").join("c");
    let nested_str = nested.to_string_lossy().to_string();

    let resolved = resolve_download_directory_from_inputs(Some(&nested_str), None)
        .expect("expected missing directory to be created");

    assert!(resolved.is_dir(), "resolved path must be a directory");
    assert!(nested.is_dir(), "nested path should be created");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn resolve_download_directory_fails_when_target_is_a_file() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("romm-dl-file-target-{ts}"));
    std::fs::create_dir_all(&base).expect("create base dir");
    let file_path = base.join("not-a-dir.txt");
    std::fs::write(&file_path, b"x").expect("create file");
    let input = file_path.to_string_lossy().to_string();

    let err = resolve_download_directory_from_inputs(Some(&input), None)
        .expect_err("file target must fail");
    assert!(
        err.to_string().contains("not a directory"),
        "unexpected error: {err:#}"
    );

    let _ = std::fs::remove_file(&file_path);
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn resolve_download_directory_env_override_takes_precedence() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let configured = std::env::temp_dir().join(format!("romm-dl-configured-{ts}"));
    let env_dir = std::env::temp_dir().join(format!("romm-dl-env-{ts}"));
    let configured_str = configured.to_string_lossy().to_string();
    let env_str = env_dir.to_string_lossy().to_string();

    let resolved = resolve_download_directory_from_inputs(Some(&configured_str), Some(&env_str))
        .expect("env override should be used");

    assert_eq!(resolved, env_dir);
    assert!(env_dir.is_dir(), "env directory should be created");
    assert!(
        !configured.is_dir(),
        "configured path should be ignored when env override is set"
    );
    let _ = std::fs::remove_dir_all(&env_dir);
}

#[test]
fn resolve_console_roms_dir_uses_platform_slug_subfolder_by_default() {
    let rom = rom_fixture_with_platform(Some("switch"), "game.zip");
    let layout = RomsLayoutConfig::default();
    let dir = resolve_console_roms_dir(&layout, Path::new("/roms"), &rom).unwrap();
    assert_eq!(dir, PathBuf::from("/roms/switch"));
}

#[test]
fn resolve_console_roms_dir_uses_custom_mapped_path() {
    let rom = rom_fixture_with_platform(Some("switch"), "game.zip");
    let mut layout = RomsLayoutConfig::default();
    let custom = std::env::temp_dir().join(format!(
        "romm-cli-manual-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&custom).unwrap();
    layout
        .platform_dirs
        .insert(rom.platform_id, custom.display().to_string());

    let dir = resolve_console_roms_dir(&layout, Path::new("/roms"), &rom).unwrap();
    assert_eq!(dir, custom);
    let _ = std::fs::remove_dir_all(custom);
}

#[test]
fn resolve_console_roms_dir_falls_back_for_unmapped_platform() {
    let rom = rom_fixture_with_platform(Some("switch"), "game.zip");
    let layout = RomsLayoutConfig::default();
    let dir = resolve_console_roms_dir(&layout, Path::new("/roms"), &rom).unwrap();
    assert_eq!(dir, PathBuf::from("/roms/switch"));
}

#[test]
fn resolve_console_save_dir_uses_platform_slug_subfolder_by_default() {
    let save_sync = SaveSyncConfig::default();
    let dir = resolve_console_save_dir(
        &save_sync,
        Path::new("/saves"),
        7,
        Some("switch"),
        Some("nintendo-switch"),
    )
    .unwrap();
    assert_eq!(dir, PathBuf::from("/saves/switch"));
}

#[test]
fn resolve_console_save_dir_uses_custom_mapped_path() {
    let mut save_sync = SaveSyncConfig::default();
    let custom = std::env::temp_dir().join(format!(
        "romm-cli-save-custom-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&custom).unwrap();
    save_sync
        .platform_dirs
        .insert(7, custom.display().to_string());

    let dir =
        resolve_console_save_dir(&save_sync, Path::new("/saves"), 7, Some("switch"), None).unwrap();
    assert_eq!(dir, custom);
    let _ = std::fs::remove_dir_all(custom);
}

#[test]
fn resolve_console_save_dir_ignores_empty_override() {
    let mut save_sync = SaveSyncConfig::default();
    save_sync.platform_dirs.insert(7, "   ".to_string());
    let dir =
        resolve_console_save_dir(&save_sync, Path::new("/saves"), 7, Some("switch"), None).unwrap();
    assert_eq!(dir, PathBuf::from("/saves/switch"));
}

#[test]
fn resolve_game_save_dir_appends_game_folder() {
    let rom = rom_fixture_with_platform(Some("switch"), "game.zip");
    let cfg = Config {
        base_url: "http://example.test".into(),
        download_dir: "/roms".into(),
        use_https: false,
        auth: None,
        extras_defaults: Default::default(),
        save_sync: SaveSyncConfig {
            save_dir: Some("/saves".into()),
            device_id: None,
            platform_dirs: HashMap::new(),
        },
        roms_layout: Default::default(),
        theme: default_theme_id(),
    };
    let dir = resolve_game_save_dir(&cfg, &rom).unwrap();
    assert_eq!(dir, PathBuf::from("/saves/switch/Game"));
}

#[test]
fn final_download_path_uses_console_folder_and_original_file_name() {
    let rom = rom_fixture_with_platform(Some("switch"), "Zelda (USA).xci");
    let base = PathBuf::from("/roms");
    let out = final_download_path_for_rom(&base, &rom);
    assert_eq!(out, PathBuf::from("/roms/switch/Zelda _USA_.xci"));
}

#[test]
fn rom_file_download_candidates_use_official_romsfiles_endpoint() {
    let target = DownloadTarget {
        kind: DownloadAssetKind::RomFile,
        title: "Update".into(),
        source_url: "/api/roms/11/files/content/update%2Ensp".into(),
        source_query: Vec::new(),
        destination: PathBuf::from("/tmp/update.nsp"),
        expected_size_bytes: Some(11),
    };

    assert_eq!(
        candidate_download_urls(&target),
        vec![
            "/api/roms/11/files/content/update%2Ensp".to_string(),
            "/api/romsfiles/11/content/update%2Ensp".to_string(),
            "/api/roms/files/11/content/update%2Ensp".to_string()
        ]
    );
}

#[test]
fn romsfiles_candidate_falls_forward_to_current_official_path() {
    let target = DownloadTarget {
        kind: DownloadAssetKind::RomFile,
        title: "Update".into(),
        source_url: "/api/romsfiles/11/content/update%2Ensp".into(),
        source_query: Vec::new(),
        destination: PathBuf::from("/tmp/update.nsp"),
        expected_size_bytes: Some(11),
    };

    assert_eq!(
        candidate_download_urls(&target),
        vec![
            "/api/romsfiles/11/content/update%2Ensp".to_string(),
            "/api/roms/11/files/content/update%2Ensp".to_string(),
            "/api/roms/files/11/content/update%2Ensp".to_string()
        ]
    );
}

#[test]
fn legacy_roms_files_candidate_falls_forward_to_romsfiles() {
    let target = DownloadTarget {
        kind: DownloadAssetKind::RomFile,
        title: "Update".into(),
        source_url: "/api/roms/files/11/content/update%2Ensp".into(),
        source_query: Vec::new(),
        destination: PathBuf::from("/tmp/update.nsp"),
        expected_size_bytes: Some(11),
    };

    assert_eq!(
        candidate_download_urls(&target),
        vec![
            "/api/roms/files/11/content/update%2Ensp".to_string(),
            "/api/roms/11/files/content/update%2Ensp".to_string(),
            "/api/romsfiles/11/content/update%2Ensp".to_string()
        ]
    );
}

#[tokio::test]
async fn prepare_target_removes_oversized_stale_rom_file() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("romm-oversized-target-{ts}.nsp"));
    tokio::fs::write(&path, b"too-large").await.unwrap();

    let target = DownloadTarget {
        kind: DownloadAssetKind::RomFile,
        title: "Base".into(),
        source_url: "/api/roms/1/files/content/base.nsp".into(),
        source_query: Vec::new(),
        destination: path.clone(),
        expected_size_bytes: Some(4),
    };

    let skip = prepare_download_target_destination(&target).await.unwrap();
    assert!(!skip);
    assert!(!path.exists());
}

#[tokio::test]
async fn prepare_target_skips_exact_size_rom_file() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("romm-exact-target-{ts}.nsp"));
    tokio::fs::write(&path, b"done").await.unwrap();

    let target = DownloadTarget {
        kind: DownloadAssetKind::RomFile,
        title: "Base".into(),
        source_url: "/api/roms/1/files/content/base.nsp".into(),
        source_query: Vec::new(),
        destination: path.clone(),
        expected_size_bytes: Some(4),
    };

    let skip = prepare_download_target_destination(&target).await.unwrap();
    assert!(skip);
    assert_eq!(tokio::fs::read(&path).await.unwrap(), b"done");
    let _ = tokio::fs::remove_file(path).await;
}

#[tokio::test]
async fn base_target_prepare_skips_exact_size_file() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("romm-base-exact-{ts}"));
    let mut rom = rom_fixture_with_platform(Some("switch"), "pack.zip");
    rom.files = vec![crate::types::RomFile {
        id: 1,
        rom_id: rom.id,
        file_name: "base.nsp".into(),
        file_path: "/base.nsp".into(),
        file_size_bytes: 4,
        category: Some(crate::types::RomFileCategory::Game),
    }];
    let target = build_base_rom_file_targets(&rom, &RomsLayoutConfig::default(), base.as_path())
        .unwrap()
        .remove(0);
    tokio::fs::create_dir_all(target.destination.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&target.destination, b"done")
        .await
        .unwrap();

    let skip = prepare_download_target_destination(&target).await.unwrap();
    assert!(skip);
    assert_eq!(tokio::fs::read(&target.destination).await.unwrap(), b"done");
    let _ = tokio::fs::remove_dir_all(base).await;
}

#[tokio::test]
async fn base_target_prepare_removes_oversized_file() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("romm-base-oversized-{ts}"));
    let mut rom = rom_fixture_with_platform(Some("switch"), "pack.zip");
    rom.files = vec![crate::types::RomFile {
        id: 1,
        rom_id: rom.id,
        file_name: "base.nsp".into(),
        file_path: "/base.nsp".into(),
        file_size_bytes: 4,
        category: Some(crate::types::RomFileCategory::Game),
    }];
    let target = build_base_rom_file_targets(&rom, &RomsLayoutConfig::default(), base.as_path())
        .unwrap()
        .remove(0);
    tokio::fs::create_dir_all(target.destination.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&target.destination, b"too-large")
        .await
        .unwrap();

    let skip = prepare_download_target_destination(&target).await.unwrap();
    assert!(!skip);
    assert!(!target.destination.exists());
    let _ = tokio::fs::remove_dir_all(base).await;
}

#[tokio::test]
async fn finalize_download_skips_when_final_exists() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("romm-finalize-skip-{ts}"));
    std::fs::create_dir_all(&base).unwrap();
    let temp = base.join("temp.part");
    let final_path = base.join("final.zip");
    std::fs::write(&temp, b"temp").unwrap();
    std::fs::write(&final_path, b"existing").unwrap();

    let result = finalize_download(&temp, &final_path).await.unwrap();
    assert_eq!(result, FinalizeResult::SkippedAlreadyExists);
    assert!(
        !temp.exists(),
        "temp file should be removed when final destination exists"
    );

    let _ = std::fs::remove_file(&final_path);
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn extract_zip_archive_writes_files_to_destination() {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("romm-extract-{ts}"));
    let zip_path = base.join("sample.zip");
    let out_dir = base.join("out");
    std::fs::create_dir_all(&base).unwrap();

    let zip_file = std::fs::File::create(&zip_path).unwrap();
    let mut writer = ZipWriter::new(zip_file);
    writer
        .start_file("nested/game.rom", SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"rom-bytes").unwrap();
    writer.finish().unwrap();

    extract_zip_archive(&zip_path, &out_dir).unwrap();

    let extracted = out_dir.join("nested").join("game.rom");
    assert!(
        extracted.exists(),
        "expected extracted file at {:?}",
        extracted
    );
    let data = std::fs::read(&extracted).unwrap();
    assert_eq!(data, b"rom-bytes");

    let _ = std::fs::remove_dir_all(&base);
}
