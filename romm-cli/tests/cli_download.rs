#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::MockServer;
use std::fs;

#[tokio::test]
async fn download_single_rom_zip_happy_path() {
    let server = MockServer::start_async().await;
    let output_dir = std::env::temp_dir().join(format!(
        "romm-cli-download-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let config_dir = std::env::temp_dir().join(format!(
        "romm-cli-download-config-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&config_dir).unwrap();

    let rom_body = r#"{
        "id": 42,
        "platform_id": 1,
        "platform_slug": "nes",
        "platform_fs_slug": "nes",
        "platform_custom_name": null,
        "platform_display_name": "NES",
        "fs_name": "game.nes",
        "fs_name_no_tags": "game.nes",
        "fs_name_no_ext": "game",
        "fs_extension": ".nes",
        "fs_path": "nes/game.nes",
        "fs_size_bytes": 4,
        "name": "Test Game",
        "slug": "test-game",
        "summary": null,
        "path_cover_small": null,
        "path_cover_large": null,
        "url_cover": null,
        "has_manual": false,
        "path_manual": null,
        "url_manual": null,
        "is_unidentified": false,
        "is_identified": true,
        "files": []
    }"#;

    let _rom_mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/roms/42");
            then.status(200)
                .header("content-type", "application/json")
                .body(rom_body);
        })
        .await;

    let rom_list_body = r#"{"items":[],"total":0,"limit":9999,"offset":0}"#;

    let _related_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms")
                .query_param("search_term", "Test Game")
                .query_param("platform_ids", "1");
            then.status(200)
                .header("content-type", "application/json")
                .body(rom_list_body);
        })
        .await;

    let zip_bytes = b"fake";
    let _download_mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms/download")
                .query_param("rom_ids", "42");
            then.status(200)
                .header("content-type", "application/zip")
                .header("content-length", zip_bytes.len().to_string())
                .body(*zip_bytes);
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_TOKEN", "test-token")
        .arg("download")
        .arg("42")
        .arg("--no-extras")
        .arg("-y")
        .arg("--output")
        .arg(&output_dir);

    cmd.assert()
        .success()
        .stderr(predicates::str::contains("Saved to"));

    let saved = output_dir.join("nes").join("rom_42.zip");
    assert!(saved.exists(), "expected {:?} to exist", saved);
    assert_eq!(fs::read(saved).unwrap(), zip_bytes);

    let _ = fs::remove_dir_all(output_dir);
    let _ = fs::remove_dir_all(config_dir);
}

#[tokio::test]
async fn download_batch_paginates_all_matching_roms() {
    let server = MockServer::start_async().await;
    let output_dir = std::env::temp_dir().join(format!(
        "romm-cli-download-batch-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&output_dir).unwrap();
    let config_dir = std::env::temp_dir().join(format!(
        "romm-cli-download-batch-config-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&config_dir).unwrap();

    let first_page = r#"{
        "items": [{
            "id": 1,
            "platform_id": 7,
            "platform_slug": "nes",
            "platform_fs_slug": "nes",
            "platform_custom_name": null,
            "platform_display_name": "NES",
            "fs_name": "one.zip",
            "fs_name_no_tags": "one",
            "fs_name_no_ext": "one",
            "fs_extension": "zip",
            "fs_path": "nes/one.zip",
            "fs_size_bytes": 3,
            "name": "One",
            "slug": "one",
            "summary": null,
            "path_cover_small": null,
            "path_cover_large": null,
            "url_cover": null,
            "has_manual": false,
            "path_manual": null,
            "url_manual": null,
            "is_unidentified": false,
            "is_identified": true,
            "files": []
        }],
        "total": 2,
        "limit": 9999,
        "offset": 0
    }"#;
    let second_page = r#"{
        "items": [{
            "id": 2,
            "platform_id": 7,
            "platform_slug": "nes",
            "platform_fs_slug": "nes",
            "platform_custom_name": null,
            "platform_display_name": "NES",
            "fs_name": "two.zip",
            "fs_name_no_tags": "two",
            "fs_name_no_ext": "two",
            "fs_extension": "zip",
            "fs_path": "nes/two.zip",
            "fs_size_bytes": 3,
            "name": "Two",
            "slug": "two",
            "summary": null,
            "path_cover_small": null,
            "path_cover_large": null,
            "url_cover": null,
            "has_manual": false,
            "path_manual": null,
            "url_manual": null,
            "is_unidentified": false,
            "is_identified": true,
            "files": []
        }],
        "total": 2,
        "limit": 9999,
        "offset": 1
    }"#;

    let first_list = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms")
                .query_param("search_term", "zelda")
                .query_param("limit", "9999")
                .query_param("offset", "0");
            then.status(200)
                .header("content-type", "application/json")
                .body(first_page);
        })
        .await;
    let second_list = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms")
                .query_param("search_term", "zelda")
                .query_param("limit", "9999")
                .query_param("offset", "1");
            then.status(200)
                .header("content-type", "application/json")
                .body(second_page);
        })
        .await;
    let download_one = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms/download")
                .query_param("rom_ids", "1");
            then.status(200)
                .header("content-type", "application/zip")
                .body("one");
        })
        .await;
    let download_two = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/roms/download")
                .query_param("rom_ids", "2");
            then.status(200)
                .header("content-type", "application/zip")
                .body("two");
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .env("API_TOKEN", "test-token")
        .arg("download")
        .arg("batch")
        .arg("--search-term")
        .arg("zelda")
        .arg("--jobs")
        .arg("1")
        .arg("--output")
        .arg(&output_dir);

    cmd.assert().success();

    first_list.assert_async().await;
    second_list.assert_async().await;
    download_one.assert_async().await;
    download_two.assert_async().await;
    assert_eq!(
        fs::read(output_dir.join("nes").join("one.zip")).unwrap(),
        b"one"
    );
    assert_eq!(
        fs::read(output_dir.join("nes").join("two.zip")).unwrap(),
        b"two"
    );

    let _ = fs::remove_dir_all(output_dir);
    let _ = fs::remove_dir_all(config_dir);
}
