use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rust-cli-template"))
}

// --- 後方互換: サブコマンドなし = parse 動作 ---

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

// --- parse サブコマンド ---

#[test]
fn test_parse_subcommand_default_output() {
    cmd()
        .arg("parse")
        .assert()
        .success()
        .stdout(predicate::str::contains("IntLiteral"));
}

#[test]
fn test_parse_subcommand_json_format() {
    cmd()
        .args(&["--format", "json", "parse"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\""));
}

// --- check サブコマンド ---

#[test]
fn test_check_subcommand_valid_input() {
    cmd()
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_check_subcommand_invalid_file() {
    cmd()
        .args(&["--input", "nonexistent_file.txt", "check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_check_subcommand_has_help() {
    cmd()
        .args(&["check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("check"));
}
