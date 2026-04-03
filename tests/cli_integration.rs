use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rust-cli-template"))
}

#[test]
fn test_default_run_succeeds() {
    cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("IntLiteral"));
}

#[test]
fn test_json_format() {
    cmd()
        .args(&["--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\""));
}

#[test]
fn test_help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: rust-cli-template"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "rust-cli-template {}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn test_invalid_input_file() {
    cmd()
        .args(&["--input", "nonexistent_file.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
