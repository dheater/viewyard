use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains(
            "The refreshingly unoptimized alternative to monorepos",
        ))
        .stdout(predicates::str::contains("dynamic repository discovery"));
}

#[test]
fn test_cli_version_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("viewyard"));
}

#[test]
fn test_create_viewset_without_github_cli() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("test-viewset")
        .current_dir(temp_dir.path())
        .env("PATH", ""); // Remove PATH to ensure gh CLI is not available

    // Should fail gracefully when GitHub CLI is not available
    cmd.assert().failure().stderr(
        predicates::str::contains("Failed to check if gh CLI is installed")
            .or(predicates::str::contains("No such file or directory"))
            .or(predicates::str::contains("GitHub CLI")),
    );
}

#[test]
fn test_create_viewset_directory_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path().join("existing-viewset");
    fs::create_dir_all(&viewset_dir).unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("existing-viewset")
        .current_dir(temp_dir.path());

    // Should fail when directory already exists
    cmd.assert().failure().stderr(
        predicates::str::contains("already exists").or(predicates::str::contains("Directory")),
    );
}

#[test]
fn test_workspace_commands_outside_view() {
    let temp_dir = TempDir::new().unwrap();

    // Try to run workspace commands outside of a view directory
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status").current_dir(temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("view directory"));
}

#[test]
fn test_basic_command_structure() {
    // Test that all main commands are available
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("viewset"))
        .stdout(predicates::str::contains("view"))
        .stdout(predicates::str::contains("status"))
        .stdout(predicates::str::contains("commit-all"))
        .stdout(predicates::str::contains("push-all"));
}
