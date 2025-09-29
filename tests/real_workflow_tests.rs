use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

mod test_utils;
use test_utils::{create_viewyard_config, GitRepoSetup};

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
fn test_commands_outside_view() {
    // Test that various commands fail gracefully when not in a view directory
    let temp_dir = TempDir::new().unwrap();

    let test_cases = vec![
        (vec!["status"], "view directory"),
        (vec!["commit-all", "test message"], "view directory"),
        (vec!["push-all"], "view directory"),
        (vec!["view", "create", "test"], "viewset"),
    ];

    for (args, expected_error) in test_cases {
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        for arg in args.iter() {
            cmd.arg(arg);
        }
        cmd.current_dir(temp_dir.path());

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains(expected_error));
    }
}

#[test]
fn test_interactive_commands_outside_view() {
    // Test interactive commands that may fail due to terminal issues in tests
    let temp_dir = TempDir::new().unwrap();

    let interactive_commands = vec![
        vec!["repo", "add"],
        vec!["repo", "remove", "some-repo"],
    ];

    for args in interactive_commands {
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        for arg in args.iter() {
            cmd.arg(arg);
        }
        cmd.current_dir(temp_dir.path());

        // Accept either view-related error or terminal error
        cmd.assert().failure().stderr(
            predicates::str::contains("view")
                .or(predicates::str::contains("terminal"))
                .or(predicates::str::contains("not a terminal")),
        );
    }
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

#[test]
fn test_view_create_with_custom_directory_name() {
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path().join("custom-viewset");
    fs::create_dir_all(&viewset_dir).unwrap();

    // Set up git repository using utility
    let git_setup = GitRepoSetup::new();

    // Create repository configuration using a custom directory name
    create_viewyard_config(
        &viewset_dir,
        "upstream-repo",
        git_setup.remote_url(),
        Some("custom-clone"),
    );

    // Run `viewyard view create` within the viewset directory
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("feature-branch")
        .current_dir(&viewset_dir);

    cmd.assert().success();

    let view_dir = viewset_dir.join("feature-branch");
    let custom_repo_dir = view_dir.join("custom-clone");
    assert!(custom_repo_dir.is_dir(), "custom directory was not created");
    assert!(
        !view_dir.join("upstream-repo").exists(),
        "repository should not exist under its original name"
    );

    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&custom_repo_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "feature-branch"
    );

    let output = StdCommand::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(&custom_repo_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git remote get-url failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        git_setup.remote_url()
    );
}

#[test]
fn test_view_create_sets_upstream_for_existing_branch() {
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path().join("upstream-viewset");
    fs::create_dir_all(&viewset_dir).unwrap();

    // Set up git repository with feature branch using utility
    let git_setup = GitRepoSetup::new();
    git_setup.create_feature_branch("feature-branch");

    // Create repository configuration
    create_viewyard_config(&viewset_dir, "upstream-repo", git_setup.remote_url(), None);

    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("feature-branch")
        .current_dir(&viewset_dir);

    cmd.assert().success();

    let view_dir = viewset_dir.join("feature-branch");
    let repo_dir = view_dir.join("upstream-repo");
    assert!(repo_dir.is_dir(), "repository directory missing");

    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "feature-branch"
    );

    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "@{u}"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "expected branch to have upstream but command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "origin/feature-branch"
    );
}

#[test]
fn test_reproduce_upstream_tracking_bug() {
    // This test specifically reproduces the scenario where a branch is created
    // without proper upstream tracking, simulating the bug reported
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path().join("bug-reproduction-viewset");
    fs::create_dir_all(&viewset_dir).unwrap();

    // Set up git repository with feature branch and upstream changes using utility
    let git_setup = GitRepoSetup::new();
    git_setup.create_feature_branch("CLIENTS-420");
    git_setup.add_upstream_commits();

    // Create viewyard configuration
    create_viewyard_config(&viewset_dir, "bug-repo", git_setup.remote_url(), None);

    // Create view using viewyard
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view").arg("create").arg("CLIENTS-420").current_dir(&viewset_dir);
    cmd.assert().success();

    let view_dir = viewset_dir.join("CLIENTS-420");
    let repo_dir = view_dir.join("bug-repo");

    // Now manually break the upstream tracking to simulate the bug
    // This simulates what might happen if the tracking setup fails
    let output = StdCommand::new("git")
        .args(["branch", "--unset-upstream"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "Failed to unset upstream");

    // Verify upstream is not set
    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "@{u}"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    assert!(!output.status.success(), "Upstream should not be configured");

    // Now try git pull - this should fail with the exact error from the issue
    let output = StdCommand::new("git")
        .args(["pull"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("git pull output: {}", stderr);

    // This should reproduce the exact error message
    assert!(
        stderr.contains("There is no tracking information for the current branch"),
        "Expected the specific error message about no tracking information, got: {}",
        stderr
    );

    // Now test that the fix works - manually set upstream and try again
    let output = StdCommand::new("git")
        .args(["branch", "--set-upstream-to=origin/CLIENTS-420"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "Failed to set upstream manually");

    // Now git pull should work
    let output = StdCommand::new("git")
        .args(["pull"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git pull should work after setting upstream: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
