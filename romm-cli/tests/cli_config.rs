#![allow(deprecated)]

use assert_cmd::Command;
use std::fs;

fn test_config_dir(prefix: &str) -> std::path::PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("romm-cli-{prefix}-test-{ts}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn config_show_sources_json_reports_env_base_url() {
    let config_dir = test_config_dir("config-show");
    let config_json = r#"{
        "base_url": "https://disk.example",
        "download_dir": "/tmp/downloads",
        "use_https": true,
        "auth": null
    }"#;
    fs::write(config_dir.join("config.json"), config_json).unwrap();

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .env("API_BASE_URL", "http://from-env.test")
        .env("API_USE_HTTPS", "false")
        .args(["config", "show", "--sources", "--json"]);

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("from-env.test"));

    let _ = fs::remove_dir_all(config_dir);
}

#[test]
fn config_env_map_prints_device_id_var() {
    Command::cargo_bin("romm-cli")
        .unwrap()
        .args(["config", "env-map", "save_sync.device_id"])
        .assert()
        .success()
        .stdout(predicates::str::contains("ROMM_SAVE_SYNC_DEVICE_ID"));
}
