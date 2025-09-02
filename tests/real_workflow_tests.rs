use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Real workflow integration tests
/// These tests verify the actual user workflows work end-to-end

#[test]
fn test_create_viewset_with_github_cli_unavailable() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg("test-viewset")
        .current_dir(temp_dir.path())
        .env("PATH", ""); // Remove PATH to ensure gh CLI is not available

    // Should fail when git is not available (since we check git first now)
    cmd.assert()
        .failure()
        .stderr(
            predicates::str::contains("Git is not installed").or(predicates::str::contains(
                "Failed to check if gh CLI is installed",
            )),
        );
}

#[test]
fn test_workspace_status_outside_view() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status").current_dir(temp_dir.path());

    // Should fail gracefully when not in a view
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("view directory"));
}

#[test]
fn test_workspace_commit_all_outside_view() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("commit-all")
        .arg("test message")
        .current_dir(temp_dir.path());

    // Should fail gracefully when not in a view
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("view directory"));
}

#[test]
fn test_workspace_push_all_outside_view() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("push-all").current_dir(temp_dir.path());

    // Should fail gracefully when not in a view
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("view directory"));
}

#[test]
fn test_repo_add_outside_view() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("repo").arg("add").current_dir(temp_dir.path());

    // Should fail gracefully when not in a view (but may fail due to terminal issues in tests)
    let result = cmd.assert().failure();
    // Accept either view-related error or terminal error
    result.stderr(
        predicates::str::contains("view")
            .or(predicates::str::contains("terminal"))
            .or(predicates::str::contains("not a terminal")),
    );
}

#[test]
fn test_repo_remove_outside_view() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("repo")
        .arg("remove")
        .arg("some-repo")
        .current_dir(temp_dir.path());

    // Should fail gracefully when not in a view
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("view"));
}

#[test]
fn test_create_viewset_in_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let viewset_name = "existing-viewset";
    let viewset_path = temp_dir.path().join(viewset_name);

    // Create directory first
    fs::create_dir_all(&viewset_path).unwrap();
    fs::write(viewset_path.join("existing-file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("viewset")
        .arg("create")
        .arg(viewset_name)
        .current_dir(temp_dir.path())
        .env("PATH", ""); // Remove PATH to ensure gh CLI is not available

    // Should fail when git is not available (since we check git first now)
    cmd.assert().failure().stderr(
        predicates::str::contains("Git is not installed")
            .or(predicates::str::contains("exists"))
            .or(predicates::str::contains("already"))
            .or(predicates::str::contains("File exists")),
    );
}

#[test]
fn test_view_create_outside_viewset() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("test-view")
        .current_dir(temp_dir.path());

    // Should fail when not in a viewset directory
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Not in a viewset directory"));
}

#[test]
fn test_help_commands_work() {
    // Test that all help commands work without errors
    let help_commands = vec![
        vec!["--help"],
        vec!["viewset", "--help"],
        vec!["view", "--help"],
        vec!["status", "--help"],
        vec!["commit-all", "--help"],
        vec!["push-all", "--help"],
        vec!["rebase", "--help"],
    ];

    for args in help_commands {
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        for arg in args {
            cmd.arg(arg);
        }

        cmd.assert().success().stdout(
            predicates::str::contains("viewyard")
                .or(predicates::str::contains("USAGE"))
                .or(predicates::str::contains("OPTIONS")),
        );
    }
}

#[test]
fn test_hierarchical_view_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a mock viewset directory structure
    let viewset_dir = temp_dir.path().join("test-viewset");
    let view_dir = viewset_dir.join("test-view");
    fs::create_dir_all(&view_dir).unwrap();

    // Create mock .viewyard-repos.json file
    let repos_json = r#"[
        {
            "name": "repo1",
            "url": "git@github.com:user/repo1.git",
            "is_private": false,
            "source": "GitHub (user)"
        },
        {
            "name": "repo2",
            "url": "git@github.com:user/repo2.git",
            "is_private": false,
            "source": "GitHub (user)"
        }
    ]"#;
    fs::write(viewset_dir.join(".viewyard-repos.json"), repos_json).unwrap();

    // Create mock git repositories (just .git directories)
    let repo1_dir = view_dir.join("repo1");
    let repo2_dir = view_dir.join("repo2");
    fs::create_dir_all(&repo1_dir).unwrap();
    fs::create_dir_all(&repo2_dir).unwrap();
    fs::create_dir_all(repo1_dir.join(".git")).unwrap();
    fs::create_dir_all(repo2_dir.join(".git")).unwrap();

    // Test status command in the view directory
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status").current_dir(&view_dir);

    // Should succeed and detect the hierarchical structure
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Viewset: test-viewset"))
        .stdout(predicates::str::contains("View: test-view"));
}

#[test]
fn test_directory_without_git_repos_fails() {
    let temp_dir = TempDir::new().unwrap();

    // Create a directory with no git repositories
    let empty_dir = temp_dir.path().join("empty-dir");
    fs::create_dir_all(&empty_dir).unwrap();

    // Create some non-git directories
    fs::create_dir_all(empty_dir.join("not-a-repo")).unwrap();
    fs::write(empty_dir.join("some-file.txt"), "content").unwrap();

    // Test status command in the empty directory
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status").current_dir(&empty_dir);

    // Should fail with appropriate error message
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Not in a view directory"));
}
