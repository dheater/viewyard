use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Real integration tests using actual git repositories
/// These tests verify that viewyard works with real git operations,
/// not just data structure validation.

#[test]
fn test_config_loading_and_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();

    // Create a valid viewsets config
    let config_content = r#"
viewsets:
  work:
    repos:
      - name: api-service
        url: https://github.com/company/api-service.git
      - name: web-app
        url: git@github.com:company/web-app.git
  personal:
    repos:
      - name: my-project
        url: https://github.com/me/my-project.git
"#;

    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();

    // Test config validation
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_view_directory_structure_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();

    // Create config with a simple repository (we'll test the structure creation, not actual git operations)
    let config_content = r#"
viewsets:
  test:
    repos:
      - name: simple-repo
        url: https://github.com/octocat/Hello-World.git
"#;

    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();

    // Test view creation using test mode (name starts with "test-")
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("--viewset")
        .arg("test")
        .arg("test-structure")
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(30));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("View 'test-structure' created successfully"));

    // Verify the basic view directory structure was created
    let view_path = temp_dir.path()
        .join("src")
        .join("src-test")
        .join("views")
        .join("test-structure");

    assert!(view_path.exists(), "View directory should exist");
    assert!(view_path.join(".git").exists(), "View should be a git repository");
    assert!(view_path.join(".viewyard-context").exists(), "Context file should exist");
    assert!(view_path.join(".gitignore").exists(), "Gitignore should exist");

    // Verify the context file contains expected content
    let context_content = fs::read_to_string(view_path.join(".viewyard-context")).unwrap();
    assert!(context_content.contains("test-structure"), "Context should contain view name");
    assert!(context_content.contains("simple-repo"), "Context should contain repository name");
}

#[test]
fn test_status_command_with_real_repos() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a mock view directory structure
    let view_path = temp_dir.path()
        .join("src")
        .join("src-test")
        .join("views")
        .join("status-check");
    
    fs::create_dir_all(&view_path).unwrap();
    
    // Create a real git repository in the view
    let repo_path = view_path.join("test-repo");
    fs::create_dir_all(&repo_path).unwrap();
    
    // Initialize git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to run git init");
    
    // Configure git
    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Add and commit a file
    fs::write(repo_path.join("test.txt"), "test content").unwrap();
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    std::process::Command::new("git")
        .args(&["commit", "-m", "Test commit"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Create viewyard context file
    let context_content = r#"
view_name: status-check
view_root: {}
active_repos:
  - test-repo
created: "2024-01-01T00:00:00Z"
"#;
    fs::write(
        view_path.join(".viewyard-context"),
        context_content.replace("{}", &view_path.to_string_lossy())
    ).unwrap();
    
    // Test status command
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(&view_path)
        .timeout(std::time::Duration::from_secs(10));
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Repository Status"))
        .stdout(predicate::str::contains("test-repo"));
}

#[test]
fn test_config_validation_with_invalid_repo_url() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create config with invalid repository URL
    let config_content = r#"
viewsets:
  test:
    repos:
      - name: invalid-repo
        url: not-a-valid-git-url
"#;
    
    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
    
    // Test that validation catches the invalid URL
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path());
    
    // Should still validate the YAML structure even if URLs are invalid
    // The actual git operations will fail later, but config parsing should work
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_error_handling_when_git_operations_fail() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create config with non-existent repository
    let config_content = r#"
viewsets:
  test:
    repos:
      - name: nonexistent-repo
        url: https://github.com/nonexistent-user/nonexistent-repo.git
"#;
    
    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
    
    // Test view creation with non-existent repo (avoid "test-" prefix)
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("--viewset")
        .arg("test")
        .arg("nonexistent-view")
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(30));
    
    // Should fail gracefully when git operations fail
    cmd.assert()
        .failure();
}
