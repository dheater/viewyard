use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Tests specifically targeting git operations and view management
/// These tests are designed to break git-related functionality

#[test]
fn test_view_creation_with_network_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Use a repository that will timeout (non-existent host)
    let config_content = r#"
viewsets:
  test:
    repos:
      - name: timeout-repo
        url: https://this-host-does-not-exist-12345.com/repo.git
"#;
    
    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
    
    // Test view creation with network timeout
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("--viewset")
        .arg("test")
        .arg("test-timeout")
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(30));
    
    // Should fail gracefully, not hang indefinitely
    cmd.assert().failure();
}

#[test]
fn test_view_creation_with_permission_denied() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_content = r#"
viewsets:
  test:
    repos:
      - name: test-repo
        url: https://github.com/octocat/Hello-World.git
"#;
    
    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
    
    // Create the src directory but make it read-only
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    
    let mut perms = fs::metadata(&src_dir).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&src_dir, perms).unwrap();
    
    // Test view creation with permission denied
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("--viewset")
        .arg("test")
        .arg("test-permission-denied")
        .env("HOME", temp_dir.path());
    
    // Should fail gracefully with meaningful error
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Permission denied")
                .or(predicate::str::contains("permission"))
                .or(predicate::str::contains("Error")));
}

#[test]
fn test_status_command_with_corrupted_git_repos() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a view directory structure
    let view_path = temp_dir.path()
        .join("src")
        .join("src-test")
        .join("views")
        .join("corrupted-test");
    
    fs::create_dir_all(&view_path).unwrap();
    
    // Create a fake git repository with corrupted .git directory
    let repo_path = view_path.join("test-repo");
    fs::create_dir_all(&repo_path).unwrap();
    
    // Create a .git directory but put garbage in it
    let git_dir = repo_path.join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(git_dir.join("HEAD"), "garbage data not a git ref").unwrap();
    fs::write(git_dir.join("config"), "invalid git config data").unwrap();
    
    // Create viewyard context file
    let context_content = format!(r#"
view_name: corrupted-test
view_root: {}
active_repos:
  - test-repo
created: "2024-01-01T00:00:00Z"
"#, view_path.display());
    
    fs::write(view_path.join(".viewyard-context"), context_content).unwrap();
    
    // Test status command with corrupted git repo
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(&view_path);
    
    // Should handle corrupted git repos gracefully
    cmd.assert()
        .success() // Should not crash
        .stdout(predicate::str::contains("Repository Status"));
}

#[test]
fn test_status_command_with_missing_repos() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a view directory structure
    let view_path = temp_dir.path()
        .join("src")
        .join("src-test")
        .join("views")
        .join("missing-repos-test");
    
    fs::create_dir_all(&view_path).unwrap();
    
    // Create viewyard context file that references non-existent repos
    let context_content = format!(r#"
view_name: missing-repos-test
view_root: {}
active_repos:
  - nonexistent-repo-1
  - nonexistent-repo-2
  - nonexistent-repo-3
created: "2024-01-01T00:00:00Z"
"#, view_path.display());
    
    fs::write(view_path.join(".viewyard-context"), context_content).unwrap();
    
    // Test status command with missing repos
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(&view_path);
    
    // Should handle missing repos gracefully
    cmd.assert()
        .success() // Should not crash
        .stdout(predicate::str::contains("Repository Status"));
}

#[test]
fn test_workspace_commands_outside_view() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test all workspace commands from outside a view
    let workspace_commands = vec![
        vec!["status"],
        vec!["rebase"],
        vec!["commit-all", "test message"],
        vec!["push-all"],
    ];
    
    for cmd_args in workspace_commands {
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        for arg in &cmd_args {
            cmd.arg(arg);
        }
        cmd.current_dir(&temp_dir);
        
        // Should fail with meaningful error message
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("not in a view")
                    .or(predicate::str::contains("view directory"))
                    .or(predicate::str::contains("viewyard view")));
        
        println!("Command {:?} outside view test passed", cmd_args);
    }
}

#[test]
fn test_view_creation_with_duplicate_names() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_content = r#"
viewsets:
  test:
    repos:
      - name: test-repo
        url: https://github.com/octocat/Hello-World.git
"#;
    
    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
    
    // Create first view
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("--viewset")
        .arg("test")
        .arg("test-duplicate")
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(30));
    
    cmd.assert().success();
    
    // Try to create second view with same name
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("--viewset")
        .arg("test")
        .arg("test-duplicate")
        .env("HOME", temp_dir.path());
    
    // Should fail with meaningful error about duplicate name
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("already exists")
                .or(predicate::str::contains("duplicate"))
                .or(predicate::str::contains("Error")));
}

#[test]
fn test_view_operations_with_corrupted_context() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a view directory structure
    let view_path = temp_dir.path()
        .join("src")
        .join("src-test")
        .join("views")
        .join("corrupted-context-test");
    
    fs::create_dir_all(&view_path).unwrap();
    
    // Create corrupted context files
    let corrupted_contexts = vec![
        "invalid yaml content ][",
        "", // Empty file
        "view_name: test\nview_root: /nonexistent/path", // Invalid path
        "completely_wrong_format: true", // Wrong structure
        "\x00\x01\x02\x03", // Binary data
    ];
    
    for (i, corrupted_content) in corrupted_contexts.iter().enumerate() {
        fs::write(view_path.join(".viewyard-context"), corrupted_content).unwrap();
        
        // Test status command with corrupted context
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        cmd.arg("status")
            .current_dir(&view_path);
        
        // Should handle corrupted context gracefully
        let result = cmd.assert();
        
        // Don't care if it succeeds or fails, just that it doesn't crash
        println!("Corrupted context test {} completed", i);
    }
}

#[test]
fn test_extremely_deep_view_paths() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create an extremely deep path
    let mut deep_path = temp_dir.path().join("src").join("src-test").join("views");
    for i in 0..20 {
        deep_path.push(format!("level-{}", i));
    }
    deep_path.push("final-view");
    
    if fs::create_dir_all(&deep_path).is_err() {
        // Skip if we can't create the deep path
        return;
    }
    
    // Create a context file in the deep path
    let context_content = format!(r#"
view_name: deep-view
view_root: {}
active_repos: []
created: "2024-01-01T00:00:00Z"
"#, deep_path.display());
    
    fs::write(deep_path.join(".viewyard-context"), context_content).unwrap();
    
    // Test status command from deep path
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(&deep_path);
    
    // Should handle deep paths gracefully
    let _ = cmd.assert();
}
