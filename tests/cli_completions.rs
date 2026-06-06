#![allow(deprecated)]

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn completions_bash_includes_subcommands() {
    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.arg("completions").arg("bash");
    cmd.assert()
        .success()
        .stdout(contains("platforms"))
        .stdout(contains("download"))
        .stdout(contains("init"));
}

#[test]
fn completions_powershell_includes_command_name() {
    let mut cmd = Command::cargo_bin("romm-cli").expect("binary");
    cmd.arg("completions").arg("powershell");
    cmd.assert()
        .success()
        .stdout(contains("romm-cli"))
        .stdout(contains("platforms"));
}
