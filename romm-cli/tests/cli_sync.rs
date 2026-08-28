#![allow(deprecated)]

use assert_cmd::Command;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("romm-cli-sync-it-{nanos}-{name}"));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn write_manifest(dir: &std::path::Path, rows_json_fragment: &str) -> std::path::PathBuf {
    let manifest_path = dir.join("manifest.json");
    let body = format!(r#"{{"saves":[{}]}}"#, rows_json_fragment);
    std::fs::write(&manifest_path, body).expect("write manifest");
    manifest_path
}

#[tokio::test]
async fn sync_device_list_fails_early_when_openapi_lacks_save_sync_endpoints() {
    let server = MockServer::start_async().await;
    let _openapi = server
        .mock_async(|when, then| {
            when.method(GET).path("/openapi.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"openapi":"3.0.0","paths":{"/api/heartbeat":{"get":{"responses":{"200":{"description":"ok"}}}}}}"#);
        })
        .await;
    let devices = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/devices");
            then.status(200)
                .header("content-type", "application/json")
                .body("[]");
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args(["sync", "device", "list"]);

    cmd.assert().failure().stderr(predicates::str::contains(
        "This RomM server does not expose save-sync endpoints",
    ));
    assert_eq!(devices.hits_async().await, 0);
}

#[tokio::test]
async fn sync_push_pull_fails_early_when_openapi_lacks_save_sync_endpoints() {
    let server = MockServer::start_async().await;
    let _openapi = server
        .mock_async(|when, then| {
            when.method(GET).path("/openapi.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"openapi":"3.0.0","paths":{}}"#);
        })
        .await;
    let push_pull = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/devices/dev-1/push-pull");
            then.status(200)
                .header("content-type", "application/json")
                .body("{}");
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args(["sync", "push-pull", "dev-1"]);

    cmd.assert().failure().stderr(predicates::str::contains(
        "This RomM server does not expose save-sync endpoints",
    ));
    assert_eq!(push_pull.hits_async().await, 0);
}

#[tokio::test]
async fn sync_plan_prints_negotiate_response() {
    let server = MockServer::start_async().await;
    let work = temp_dir("plan");
    let save_path = work.join("game.sav");
    std::fs::write(&save_path, b"save-data").expect("write save");
    let manifest = write_manifest(
        &work,
        r#"{"rom_id":7,"path":"game.sav","emulator":"retroarch"}"#,
    );

    let _m = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/negotiate");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session_id": 42,
                        "operations": [],
                        "total_upload": 0,
                        "total_download": 0,
                        "total_conflict": 0,
                        "total_no_op": 0
                    }"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .arg("sync")
        .arg("plan")
        .arg("--device-id")
        .arg("dev-1")
        .arg("--manifest")
        .arg(manifest);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"session_id\": 42"));
}

#[tokio::test]
async fn sync_run_uploads_and_completes() {
    let server = MockServer::start_async().await;
    let work = temp_dir("run-upload");
    let save_path = work.join("upload.sav");
    std::fs::write(&save_path, b"upload-me").expect("write save");
    let manifest = write_manifest(
        &work,
        r#"{"rom_id":7,"path":"upload.sav","file_name":"remote.sav","slot":"slot1","emulator":"retroarch"}"#,
    );

    let _negotiate = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/negotiate");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session_id": 11,
                        "operations": [{
                            "action":"upload",
                            "rom_id":7,
                            "save_id":null,
                            "file_name":"remote.sav",
                            "slot":"slot1",
                            "emulator":"retroarch",
                            "reason":"client newer",
                            "server_updated_at":null,
                            "server_content_hash":null
                        }],
                        "total_upload": 1,
                        "total_download": 0,
                        "total_conflict": 0,
                        "total_no_op": 0
                    }"#,
                );
        })
        .await;

    let upload = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/api/saves")
                .query_param("rom_id", "7")
                .query_param("device_id", "dev-1")
                .query_param("session_id", "11")
                .query_param("slot", "slot1")
                .query_param("emulator", "retroarch")
                .body_contains("filename=\"remote.sav\"");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"id": 1}"#);
        })
        .await;

    let complete = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/sessions/11/complete");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session": {
                            "id": 11,
                            "device_id": "dev-1",
                            "user_id": 1,
                            "status": "COMPLETED",
                            "initiated_at": "2026-01-01T00:00:00Z",
                            "completed_at": "2026-01-01T00:00:01Z",
                            "operations_planned": 1,
                            "operations_completed": 1,
                            "operations_failed": 0,
                            "error_message": null,
                            "created_at": "2026-01-01T00:00:00Z",
                            "updated_at": "2026-01-01T00:00:01Z"
                        },
                        "play_session_ingest": null
                    }"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args([
            "sync",
            "run",
            "--device-id",
            "dev-1",
            "--manifest",
            manifest.to_str().expect("manifest str"),
        ]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("uploaded=1"));
    upload.assert();
    complete.assert();
}

#[tokio::test]
async fn sync_run_downloads_file() {
    let server = MockServer::start_async().await;
    let work = temp_dir("run-download");
    let download_dir = work.join("downloads");
    std::fs::create_dir_all(&download_dir).expect("mkdir");
    let manifest = write_manifest(&work, "");

    let _negotiate = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/negotiate");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session_id": 12,
                        "operations": [{
                            "action":"download",
                            "rom_id":8,
                            "save_id":55,
                            "file_name":"from-server.sav",
                            "slot":null,
                            "emulator":null,
                            "reason":"server newer",
                            "server_updated_at":"2026-01-01T00:00:00Z",
                            "server_content_hash":"abc"
                        }],
                        "total_upload": 0,
                        "total_download": 1,
                        "total_conflict": 0,
                        "total_no_op": 0
                    }"#,
                );
        })
        .await;

    let _download = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/api/saves/55/content")
                .query_param("device_id", "dev-2")
                .query_param("session_id", "12");
            then.status(200).body("downloaded-bytes");
        })
        .await;

    let complete = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/sessions/12/complete");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session": {
                            "id": 12,
                            "device_id": "dev-2",
                            "user_id": 1,
                            "status": "COMPLETED",
                            "initiated_at": "2026-01-01T00:00:00Z",
                            "completed_at": "2026-01-01T00:00:01Z",
                            "operations_planned": 1,
                            "operations_completed": 1,
                            "operations_failed": 0,
                            "error_message": null,
                            "created_at": "2026-01-01T00:00:00Z",
                            "updated_at": "2026-01-01T00:00:01Z"
                        },
                        "play_session_ingest": null
                    }"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args([
            "sync",
            "run",
            "--device-id",
            "dev-2",
            "--manifest",
            manifest.to_str().expect("manifest str"),
            "--download-dir",
            download_dir.to_str().expect("download dir str"),
        ]);

    cmd.assert().success();
    complete.assert();
    let downloaded = download_dir.join("from-server.sav");
    assert!(downloaded.exists(), "expected downloaded file");
}

#[tokio::test]
async fn sync_run_conflict_fails_by_default() {
    let server = MockServer::start_async().await;
    let work = temp_dir("run-conflict-fail");
    let manifest = write_manifest(&work, "");

    let _negotiate = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/negotiate");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session_id": 13,
                        "operations": [{
                            "action":"conflict",
                            "rom_id":9,
                            "save_id":2,
                            "file_name":"conflict.sav",
                            "slot":null,
                            "emulator":null,
                            "reason":"Both sides changed",
                            "server_updated_at":"2026-01-01T00:00:00Z",
                            "server_content_hash":"abc"
                        }],
                        "total_upload": 0,
                        "total_download": 0,
                        "total_conflict": 1,
                        "total_no_op": 0
                    }"#,
                );
        })
        .await;

    let complete = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/sessions/13/complete");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session": {
                            "id": 13,
                            "device_id": "dev-3",
                            "user_id": 1,
                            "status": "COMPLETED",
                            "initiated_at": "2026-01-01T00:00:00Z",
                            "completed_at": "2026-01-01T00:00:01Z",
                            "operations_planned": 1,
                            "operations_completed": 0,
                            "operations_failed": 1,
                            "error_message": null,
                            "created_at": "2026-01-01T00:00:00Z",
                            "updated_at": "2026-01-01T00:00:01Z"
                        },
                        "play_session_ingest": null
                    }"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args([
            "sync",
            "run",
            "--device-id",
            "dev-3",
            "--manifest",
            manifest.to_str().expect("manifest str"),
        ]);

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Conflict for rom_id=9"));
    complete.assert();
}

#[tokio::test]
async fn sync_run_conflict_skip_succeeds() {
    let server = MockServer::start_async().await;
    let work = temp_dir("run-conflict-skip");
    let manifest = write_manifest(&work, "");

    let _negotiate = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/negotiate");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session_id": 14,
                        "operations": [{
                            "action":"conflict",
                            "rom_id":10,
                            "save_id":3,
                            "file_name":"conflict2.sav",
                            "slot":null,
                            "emulator":null,
                            "reason":"Both sides changed",
                            "server_updated_at":"2026-01-01T00:00:00Z",
                            "server_content_hash":"abc"
                        }],
                        "total_upload": 0,
                        "total_download": 0,
                        "total_conflict": 1,
                        "total_no_op": 0
                    }"#,
                );
        })
        .await;

    let complete = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/sync/sessions/14/complete");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                        "session": {
                            "id": 14,
                            "device_id": "dev-4",
                            "user_id": 1,
                            "status": "COMPLETED",
                            "initiated_at": "2026-01-01T00:00:00Z",
                            "completed_at": "2026-01-01T00:00:01Z",
                            "operations_planned": 1,
                            "operations_completed": 0,
                            "operations_failed": 0,
                            "error_message": null,
                            "created_at": "2026-01-01T00:00:00Z",
                            "updated_at": "2026-01-01T00:00:01Z"
                        },
                        "play_session_ingest": null
                    }"#,
                );
        })
        .await;

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("API_BASE_URL", server.base_url())
        .env("API_USE_HTTPS", "false")
        .args([
            "sync",
            "run",
            "--device-id",
            "dev-4",
            "--manifest",
            manifest.to_str().expect("manifest str"),
            "--conflict",
            "skip",
        ]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("conflicts_skipped=1"));
    complete.assert();
}
