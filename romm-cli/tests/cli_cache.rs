#![allow(deprecated)]

use assert_cmd::Command;
use predicates::str::contains;

fn unique_temp_file(name: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{ts}.json"))
}

#[test]
fn cache_path_prints_effective_path_without_api_config() {
    let path = unique_temp_file("romm-cache-path");
    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_CACHE_PATH", &path).arg("cache").arg("path");
    cmd.assert()
        .success()
        .stdout(contains(path.to_string_lossy().as_ref()));
}

#[test]
fn cache_info_reports_missing_file() {
    let path = unique_temp_file("romm-cache-info");
    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_CACHE_PATH", &path).arg("cache").arg("info");
    cmd.assert()
        .success()
        .stdout(contains("exists: false"))
        .stdout(contains(path.to_string_lossy().as_ref()));
}

#[test]
fn cache_clear_removes_existing_file() {
    let path = unique_temp_file("romm-cache-clear");
    std::fs::write(&path, "{}").expect("seed cache file");

    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.env("ROMM_CACHE_PATH", &path).arg("cache").arg("clear");
    cmd.assert()
        .success()
        .stdout(contains("ROM cache cleared."));
    assert!(!path.exists(), "cache file should be removed");
}
