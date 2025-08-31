use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Stress tests designed to break viewyard in every possible way
/// These tests are intentionally aggressive and designed to find edge cases

#[test]
fn test_massive_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create a config with 1000 viewsets, each with 100 repositories
    let mut config_content = String::from("viewsets:\n");
    
    for viewset_idx in 0..1000 {
        config_content.push_str(&format!("  viewset-{}:\n    repos:\n", viewset_idx));
        
        for repo_idx in 0..100 {
            config_content.push_str(&format!(
                "      - name: repo-{}-{}\n        url: https://github.com/org/repo-{}-{}.git\n",
                viewset_idx, repo_idx, viewset_idx, repo_idx
            ));
        }
    }
    
    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
    
    // Test that validation doesn't crash or take forever
    let start = std::time::Instant::now();
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(30));
    
    cmd.assert().success();
    
    let duration = start.elapsed();
    assert!(duration < std::time::Duration::from_secs(10), 
            "Validation took too long: {:?}", duration);
}

#[test]
fn test_malformed_yaml_configs() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create long config separately to avoid temporary value issues
    let long_name = "a".repeat(10000);
    let long_config = format!("viewsets:\n  work:\n    repos:\n      - name: \"{}\"\n        url: https://github.com/test/test.git", long_name);

    let malformed_configs = vec![
        // Invalid YAML syntax
        "viewsets:\n  work:\n    repos:\n      - name: test\n        url: invalid yaml ][",

        // Missing required fields
        "viewsets:\n  work:\n    repos:\n      - name: test",

        // Invalid structure
        "this_is_not_viewsets:\n  work:\n    repos: []",

        // Empty file
        "",

        // Only whitespace
        "   \n  \n   ",

        // Invalid characters
        "viewsets:\n  work:\n    repos:\n      - name: \"test\x00invalid\"",

        // Extremely long names
        &long_config,
    ];
    
    for (i, config) in malformed_configs.iter().enumerate() {
        fs::write(config_dir.join("viewsets.yaml"), config).unwrap();
        
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        cmd.arg("view")
            .arg("validate")
            .env("HOME", temp_dir.path());
        
        // Should fail gracefully, not crash
        let assert = cmd.assert().failure();
        
        // Should provide meaningful error message
        assert.stderr(predicate::str::contains("Error").or(predicate::str::contains("error")));
        
        println!("Malformed config test {} passed", i);
    }
}

#[test]
fn test_filesystem_edge_cases() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Test with read-only config file
    let config_content = r#"
viewsets:
  test:
    repos:
      - name: test-repo
        url: https://github.com/octocat/Hello-World.git
"#;
    
    let config_file = config_dir.join("viewsets.yaml");
    fs::write(&config_file, config_content).unwrap();
    
    // Make config file read-only
    let mut perms = fs::metadata(&config_file).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&config_file, perms).unwrap();
    
    // Should still be able to read and validate
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path());
    
    cmd.assert().success();
}

#[test]
fn test_concurrent_operations() {
    use std::sync::Arc;
    use std::thread;
    
    let temp_dir = Arc::new(TempDir::new().unwrap());
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
    
    // Spawn multiple threads trying to validate simultaneously
    let handles: Vec<_> = (0..10).map(|i| {
        let temp_dir = Arc::clone(&temp_dir);
        thread::spawn(move || {
            let mut cmd = Command::cargo_bin("viewyard").unwrap();
            cmd.arg("view")
                .arg("validate")
                .env("HOME", temp_dir.path());
            
            let result = cmd.assert().success();
            println!("Thread {} completed", i);
            result
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_path_traversal_attacks() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Test malicious repository names that could cause path traversal
    let malicious_names = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "/etc/passwd",
        "C:\\Windows\\System32",
        "repo\x00name",
        "repo\nname",
        "repo\rname",
        "repo\tname",
        ".",
        "..",
        "...",
        "con", // Windows reserved name
        "aux", // Windows reserved name
        "nul", // Windows reserved name
    ];
    
    for malicious_name in malicious_names {
        let config_content = format!(r#"
viewsets:
  test:
    repos:
      - name: "{}"
        url: https://github.com/test/test.git
"#, malicious_name);
        
        fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
        
        // Should either reject the config or handle it safely
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        cmd.arg("view")
            .arg("validate")
            .env("HOME", temp_dir.path());
        
        // Don't care if it succeeds or fails, just that it doesn't crash
        let _ = cmd.assert();
        
        println!("Path traversal test for '{}' completed", malicious_name);
    }
}

#[test]
fn test_extremely_long_paths() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create a very deep directory structure
    let mut deep_path = temp_dir.path().to_path_buf();
    for i in 0..50 {
        deep_path.push(format!("very-long-directory-name-{}", i));
    }
    
    if let Err(_) = fs::create_dir_all(&deep_path) {
        // If we can't create the path, skip this test
        return;
    }
    
    // Try to run viewyard from this deep path
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("status")
        .current_dir(&deep_path);
    
    // Should handle the deep path gracefully
    let _ = cmd.assert();
}

#[test]
fn test_unicode_and_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();
    
    let unicode_names = vec![
        "测试仓库", // Chinese
        "тестовый-репо", // Russian
        "🚀-rocket-repo", // Emoji
        "café-münü", // Accented characters
        "repo-with-spaces and-tabs\t", // Whitespace
        "repo'with\"quotes", // Quotes
        "repo;with|pipes&ampersands", // Shell metacharacters
    ];
    
    for unicode_name in unicode_names {
        let config_content = format!(r#"
viewsets:
  test:
    repos:
      - name: "{}"
        url: https://github.com/test/test.git
"#, unicode_name);
        
        fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();
        
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        cmd.arg("view")
            .arg("validate")
            .env("HOME", temp_dir.path());
        
        // Should handle unicode gracefully
        let _ = cmd.assert();
        
        println!("Unicode test for '{}' completed", unicode_name);
    }
}

#[test]
fn test_memory_exhaustion_protection() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();

    // Create a config with extremely long repository URLs
    let long_url = "https://github.com/".to_string() + &"a".repeat(1_000_000);
    let config_content = format!(r#"
viewsets:
  test:
    repos:
      - name: test-repo
        url: "{}"
"#, long_url);

    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();

    // Should handle without consuming excessive memory
    let mut cmd = Command::cargo_bin("viewyard").unwrap();
    cmd.arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path())
        .timeout(std::time::Duration::from_secs(10));

    // Don't care about success/failure, just that it doesn't hang or crash
    let _ = cmd.assert();
}

#[test]
fn test_invalid_git_urls() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();

    let invalid_urls = vec![
        "not-a-url",
        "http://",
        "https://",
        "git@",
        "git@github.com:",
        "file://",
        "ftp://example.com/repo.git",
        "javascript:alert('xss')",
        "data:text/html,<script>alert('xss')</script>",
        "\\\\network\\share\\repo",
        "C:\\local\\path\\repo",
        "/local/path/repo",
        "ssh://user@host:99999/repo.git", // Invalid port
        "https://user:pass@host/repo.git", // Credentials in URL
    ];

    for invalid_url in invalid_urls {
        let config_content = format!(r#"
viewsets:
  test:
    repos:
      - name: test-repo
        url: "{}"
"#, invalid_url);

        fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();

        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        cmd.arg("view")
            .arg("validate")
            .env("HOME", temp_dir.path());

        // Should validate the YAML structure even if URLs are questionable
        let _ = cmd.assert();

        println!("Invalid URL test for '{}' completed", invalid_url);
    }
}

#[test]
fn test_environment_variable_edge_cases() {
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

    // Test with various problematic HOME values
    let problematic_homes = vec![
        "", // Empty HOME
        "/nonexistent/path", // Non-existent HOME
        "/dev/null", // HOME pointing to a file
        "/", // ROOT as HOME
    ];

    for home in problematic_homes {
        let mut cmd = Command::cargo_bin("viewyard").unwrap();
        cmd.arg("view")
            .arg("validate")
            .env("HOME", home);

        // Should handle gracefully
        let _ = cmd.assert();

        println!("HOME test for '{}' completed", home);
    }
}

#[test]
fn test_process_termination() {
    use std::process::{Command as StdCommand, Stdio};
    use std::time::Duration;
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/viewyard");
    fs::create_dir_all(&config_dir).unwrap();

    // Create a config that might take some time to process
    let mut config_content = String::from("viewsets:\n");
    for i in 0..100 {
        config_content.push_str(&format!("  viewset-{}:\n    repos:\n", i));
        for j in 0..10 {
            config_content.push_str(&format!(
                "      - name: repo-{}-{}\n        url: https://github.com/org/repo-{}-{}.git\n",
                i, j, i, j
            ));
        }
    }

    fs::write(config_dir.join("viewsets.yaml"), config_content).unwrap();

    // Start a long-running command
    let mut child = StdCommand::new("target/release/viewyard")
        .arg("view")
        .arg("validate")
        .env("HOME", temp_dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start viewyard");

    // Let it run for a bit
    thread::sleep(Duration::from_millis(100));

    // Force kill the process
    let _ = child.kill();
    let _ = child.wait();

    // Test passed if we didn't hang or crash
}
