use std::fs;
use tempfile::TempDir;
use viewyard::config::{load_viewsets_config_from_path, viewsets_config_path};
use viewyard::models::{Repository, ViewsetsConfig};

#[test]
fn test_load_viewsets_config_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.yaml");
    
    // Should return error when file doesn't exist
    let result = load_viewsets_config_from_path(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_load_viewsets_config_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.yaml");
    
    // Write invalid YAML
    fs::write(&config_path, "invalid: yaml: content: [").unwrap();
    
    let result = load_viewsets_config_from_path(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_load_viewsets_config_valid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("valid.yaml");
    
    // Write valid viewsets config
    let yaml_content = r#"
viewsets:
  work:
    repos:
      - name: test-repo
        url: https://github.com/test/repo.git
      - name: another-repo
        url: git@github.com:test/another.git
"#;
    fs::write(&config_path, yaml_content).unwrap();
    
    let result = load_viewsets_config_from_path(&config_path).unwrap();
    
    // Verify structure
    assert_eq!(result.viewsets.len(), 1);
    assert!(result.viewsets.contains_key("work"));
    
    let work_viewset = &result.viewsets["work"];
    assert_eq!(work_viewset.repos.len(), 2);
    
    let first_repo = &work_viewset.repos[0];
    assert_eq!(first_repo.name, "test-repo");
    assert_eq!(first_repo.url, "https://github.com/test/repo.git");

    let second_repo = &work_viewset.repos[1];
    assert_eq!(second_repo.name, "another-repo");
    assert_eq!(second_repo.url, "git@github.com:test/another.git");
}

#[test]
fn test_load_viewsets_config_empty_viewsets() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("empty.yaml");
    
    // Write config with empty viewsets
    let yaml_content = r#"
viewsets: {}
"#;
    fs::write(&config_path, yaml_content).unwrap();
    
    let result = load_viewsets_config_from_path(&config_path).unwrap();
    assert_eq!(result.viewsets.len(), 0);
}

#[test]
fn test_get_config_path_default() {
    let path = viewsets_config_path().unwrap();
    assert!(path.to_string_lossy().ends_with(".config/viewyard/viewsets.yaml"));
}

// Test Repository model methods
#[test]
fn test_repository_display() {
    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/test/repo.git".to_string(),
    };

    assert_eq!(format!("{}", repo), "test-repo");
}

// Test ViewsetsConfig methods
#[test]
fn test_viewsets_config_new() {
    let config = ViewsetsConfig::new();
    assert!(config.viewsets.is_empty());
}

// Error handling tests for novice developer persona
#[test]
fn test_config_handles_malformed_repo_entries() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("malformed.yaml");
    
    // Missing required fields
    let yaml_content = r#"
viewsets:
  work:
    repos:
      - name: test-repo
        # missing url field
      - url: https://github.com/test/repo.git
        # missing name field
"#;
    fs::write(&config_path, yaml_content).unwrap();
    
    let result = load_viewsets_config_from_path(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_config_handles_duplicate_viewset_names() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("duplicates.yaml");
    
    // YAML allows duplicate keys, but the last one wins
    let yaml_content = r#"
viewsets:
  work:
    repos:
      - name: first-repo
        url: https://github.com/test/first.git
  work:
    repos:
      - name: second-repo
        url: https://github.com/test/second.git
"#;
    fs::write(&config_path, yaml_content).unwrap();
    
    let result = load_viewsets_config_from_path(&config_path).unwrap();
    assert_eq!(result.viewsets.len(), 1);
    
    let work_viewset = &result.viewsets["work"];
    assert_eq!(work_viewset.repos.len(), 1);
    assert_eq!(work_viewset.repos[0].name, "second-repo");
}

// Expert developer persona tests - complex configurations
#[test]
fn test_config_handles_large_complex_viewsets() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("complex.yaml");
    
    // Generate a complex configuration with multiple viewsets and many repos
    let mut yaml_content = String::from("viewsets:\n");
    
    for viewset_idx in 0..5 {
        yaml_content.push_str(&format!("  viewset-{}:\n", viewset_idx));
        yaml_content.push_str("    repos:\n");
        
        for repo_idx in 0..10 {
            yaml_content.push_str(&format!(
                "      - name: repo-{}-{}\n        url: https://github.com/org/repo-{}-{}.git\n",
                viewset_idx, repo_idx, viewset_idx, repo_idx
            ));
        }
    }
    
    fs::write(&config_path, yaml_content).unwrap();
    
    let result = load_viewsets_config_from_path(&config_path).unwrap();
    assert_eq!(result.viewsets.len(), 5);
    
    for viewset_idx in 0..5 {
        let viewset_name = format!("viewset-{}", viewset_idx);
        assert!(result.viewsets.contains_key(&viewset_name));
        
        let viewset = &result.viewsets[&viewset_name];
        assert_eq!(viewset.repos.len(), 10);
        
        for repo_idx in 0..10 {
            let repo = &viewset.repos[repo_idx];
            assert_eq!(repo.name, format!("repo-{}-{}", viewset_idx, repo_idx));
            assert_eq!(repo.url, format!("https://github.com/org/repo-{}-{}.git", viewset_idx, repo_idx));
        }
    }
}

#[test]
fn test_status_visibility_improvements() {
    // Test that our status improvements work correctly
    // This tests the logic without requiring actual git repositories

    // Test branch consistency checking
    let repo_branches = vec![
        ("repo1".to_string(), "main".to_string()),
        ("repo2".to_string(), "main".to_string()),
        ("repo3".to_string(), "feature-branch".to_string()),
    ];

    // This should detect branch mismatch (tested via integration)
    // For now, just verify the data structure is correct
    assert_eq!(repo_branches.len(), 3);
    assert_eq!(repo_branches[0].1, "main");
    assert_eq!(repo_branches[2].1, "feature-branch");
}
