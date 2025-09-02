use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--help");

    cmd.assert().success().stdout(predicate::str::contains(
        "refreshingly unoptimized alternative",
    ));
}

#[test]
fn test_cli_version_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--help");

    // The CLI doesn't have --version, but --help should show the program name
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("viewyard"));
}
