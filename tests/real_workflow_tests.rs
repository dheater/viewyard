use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use std::fs;
use std::process::Command as StdCommand;
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

#[test]
fn test_view_create_with_custom_directory_name() {
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path().join("custom-viewset");
    fs::create_dir_all(&viewset_dir).unwrap();

    // Prepare a bare remote repository with an initial commit
    let remote_dir = TempDir::new().unwrap();
    let remote_path = remote_dir.path();

    let output = StdCommand::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init --bare failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let seed_dir = TempDir::new().unwrap();
    let remote_path_str = remote_path.to_str().unwrap();
    let output = StdCommand::new("git")
        .args(["clone", remote_path_str, "."])
        .current_dir(seed_dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(seed_dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git config user.name failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(seed_dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git config user.email failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::write(
        seed_dir.path().join("README.md"),
        "# Custom Directory Test\n",
    )
    .unwrap();

    let output = StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(seed_dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(seed_dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["branch", "-M", "main"])
        .current_dir(seed_dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git branch -M main failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(seed_dir.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git push failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(remote_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git symbolic-ref on remote failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Create repository configuration using a custom directory name
    let repos = json!([
        {
            "name": "upstream-repo",
            "url": remote_path_str,
            "is_private": false,
            "source": "GitHub (test-user)",
            "account": "test-user",
            "directory_name": "custom-clone"
        }
    ]);
    fs::write(
        viewset_dir.join(".viewyard-repos.json"),
        serde_json::to_string_pretty(&repos).unwrap(),
    )
    .unwrap();

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
        remote_path_str
    );
}

#[test]
fn test_view_create_sets_upstream_for_existing_branch() {
    let temp_dir = TempDir::new().unwrap();
    let viewset_dir = temp_dir.path().join("upstream-viewset");
    fs::create_dir_all(&viewset_dir).unwrap();

    // Prepare a bare remote repository with an existing feature branch
    let remote_dir = TempDir::new().unwrap();
    let remote_path = remote_dir.path();

    let output = StdCommand::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init --bare failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let seed_dir = TempDir::new().unwrap();
    let seed_path = seed_dir.path();
    let remote_path_str = remote_path.to_str().unwrap();

    let output = StdCommand::new("git")
        .args(["init"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["remote", "add", "origin", remote_path_str])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git remote add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git config user.name failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git config user.email failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::write(seed_path.join("README.md"), "# Upstream Test\n").unwrap();

    let output = StdCommand::new("git")
        .args(["add", "README.md"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["branch", "-M", "main"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git branch -M main failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git push main failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["checkout", "-b", "feature-branch"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git checkout -b feature-branch failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["push", "-u", "origin", "feature-branch"])
        .current_dir(seed_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git push feature-branch failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = StdCommand::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(remote_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git symbolic-ref on remote failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Create repository configuration
    let repos = json!([
        {
            "name": "upstream-repo",
            "url": remote_path_str,
            "is_private": false,
            "source": "GitHub (test-user)",
            "account": "test-user"
        }
    ]);
    fs::write(
        viewset_dir.join(".viewyard-repos.json"),
        serde_json::to_string_pretty(&repos).unwrap(),
    )
    .unwrap();

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
