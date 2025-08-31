use assert_cmd::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// Helper function to create a test viewsets config
fn create_test_config(temp_dir: &TempDir) -> String {
    let yaml_content = r#"
viewsets:
  work:
    repos:
      - name: test-repo
        url: https://github.com/test/repo.git
      - name: another-repo
        url: git@github.com:test/another.git
        build: cargo build
        test: cargo test
  personal:
    repos:
      - name: personal-project
        url: https://github.com/me/project.git
"#;
    yaml_content.to_string()
}

// Helper function to create a mock git repository
fn create_mock_git_repo(path: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join(".git"))?;
    fs::write(path.join(".git/config"), "[core]\n\trepositoryformatversion = 0")?;
    fs::write(path.join("README.md"), "# Test Repository")?;
    Ok(())
}

#[test]
fn test_cli_help_command() {
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Multi-repository workspace management tool"));
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
fn test_view_list_no_config() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("list")
        .env("HOME", temp_dir.path());
    
    // Should fail when no config exists
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Viewsets configuration not found"));
}

#[test]
fn test_view_validate_with_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_path = config_dir.join("viewsets.yaml");
    let yaml_content = r#"
viewsets:
  work:
    repos:
      - name: valid-repo
        url: https://github.com/test/repo.git
"#;
    fs::write(&config_path, yaml_content).unwrap();
    
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Configuration is valid"));
}

#[test]
fn test_view_validate_with_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_path = config_dir.join("viewsets.yaml");
    let yaml_content = "invalid: yaml: content: [";
    fs::write(&config_path, yaml_content).unwrap();
    
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path());
    
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Failed to parse config file"));
}

// Integration test for novice developer persona - error handling
#[test]
fn test_novice_developer_wrong_directory() {
    let temp_dir = TempDir::new().unwrap();
    
    // Try to run workspace commands outside of a view directory
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(temp_dir.path());
    
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("not in a view directory").or(
            predicates::str::contains("Could not detect current view")
        ));
}

#[test]
fn test_novice_developer_invalid_repo_name() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_path = config_dir.join("viewsets.yaml");
    fs::write(&config_path, create_test_config(&temp_dir)).unwrap();
    
    // Try to create a view with invalid characters
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("create")
        .arg("invalid/name with spaces!")
        .env("HOME", temp_dir.path());
    
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Invalid").or(
            predicates::str::contains("name")
        ));
}

// Integration test for expert developer persona - complex workflows
#[test]
fn test_expert_developer_multiple_viewsets() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create complex config with multiple viewsets
    let config_path = config_dir.join("viewsets.yaml");
    let yaml_content = r#"
viewsets:
  work:
    repos:
      - name: service-a
        url: https://github.com/company/service-a.git
      - name: service-b
        url: https://github.com/company/service-b.git
      - name: shared-lib
        url: https://github.com/company/shared-lib.git
  personal:
    repos:
      - name: my-project
        url: https://github.com/me/project.git
      - name: experiments
        url: https://github.com/me/experiments.git
  client-work:
    repos:
      - name: client-app
        url: https://github.com/client/app.git
      - name: client-api
        url: https://github.com/client/api.git
"#;
    fs::write(&config_path, yaml_content).unwrap();
    
    // List views should show all viewsets
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("list")
        .env("HOME", temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("work"))
        .stdout(predicates::str::contains("personal"))
        .stdout(predicates::str::contains("client-work"));
}

#[test]
fn test_workspace_status_outside_view() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(temp_dir.path());
    
    // Should fail gracefully when not in a view
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("view").or(
            predicates::str::contains("directory")
        ));
}

// Error recovery test
#[test]
fn test_error_recovery_corrupted_view() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_path = config_dir.join("viewsets.yaml");
    fs::write(&config_path, create_test_config(&temp_dir)).unwrap();
    
    // Create a view directory structure but corrupt it
    let view_dir = temp_dir.path().join("src-work/views/test-view");
    fs::create_dir_all(&view_dir).unwrap();
    
    // Create invalid .viewyard-context file
    fs::write(view_dir.join(".viewyard-context"), "corrupted content").unwrap();
    
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(&view_dir);
    
    // Should handle corrupted context gracefully
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("context").or(
            predicates::str::contains("corrupted").or(
                predicates::str::contains("invalid")
            )
        ));
}

// Performance test for expert developer - large configuration
#[test]
fn test_large_configuration_performance() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Generate large configuration
    let mut yaml_content = String::from("viewsets:\n");
    for viewset_idx in 0..20 {
        yaml_content.push_str(&format!("  large-viewset-{}:\n", viewset_idx));
        yaml_content.push_str("    repos:\n");
        for repo_idx in 0..50 {
            yaml_content.push_str(&format!(
                "      - name: repo-{}-{}\n        url: https://github.com/org/repo-{}-{}.git\n",
                viewset_idx, repo_idx, viewset_idx, repo_idx
            ));
        }
    }
    
    let config_path = config_dir.join("viewsets.yaml");
    fs::write(&config_path, yaml_content).unwrap();
    
    // Validate should complete in reasonable time
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(10)); // Should complete within 10 seconds
    
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Configuration is valid"));
}
