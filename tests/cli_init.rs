#![allow(deprecated)]

use assert_cmd::Command;
use std::fs;

#[test]
fn init_requires_token_with_url() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config_dir = std::env::temp_dir().join(format!("romm-cli-test-1-{}", ts));

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .arg("init")
        .arg("--url")
        .arg("https://romm.example.com");

    cmd.assert().failure().stderr(predicates::str::contains(
        "--url requires either --token or --token-file",
    ));
}

#[test]
fn init_requires_url_with_token() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config_dir = std::env::temp_dir().join(format!("romm-cli-test-2-{}", ts));

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .arg("init")
        .arg("--token")
        .arg("my-secret-token");

    cmd.assert().failure().stderr(predicates::str::contains(
        "--token and --token-file require --url",
    ));
}

#[test]
fn init_non_interactive_writes_config() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let config_dir = std::env::temp_dir().join(format!("romm-cli-test-{}", ts));

    let mut cmd = Command::cargo_bin("romm-cli").unwrap();
    cmd.env("ROMM_TEST_CONFIG_DIR", config_dir.as_os_str())
        .arg("init")
        .arg("--url")
        .arg("https://romm.example.com")
        .arg("--token")
        .arg("my-secret-token")
        .arg("--download-dir")
        .arg("/tmp/roms")
        .arg("--no-https");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Wrote"));

    let env_path = config_dir.join(".env");
    assert!(env_path.exists());

    let content = fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("API_BASE_URL=https://romm.example.com"));
    assert!(content.contains("ROMM_DOWNLOAD_DIR=/tmp/roms"));
    assert!(content.contains("API_USE_HTTPS=false"));
    assert!(content.contains("Bearer token"));

    let _ = fs::remove_dir_all(config_dir);
}
