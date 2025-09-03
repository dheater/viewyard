use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::TempDir;
use viewyard::git;

/// Helper function to create a test repo with a remote that has a specific default branch
fn create_test_repo_with_remote_default(remote_default: &str) -> Result<TempDir> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create a bare repository to act as the "remote"
    let remote_dir = TempDir::new()?;
    let remote_path = remote_dir.path();

    Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_path)
        .output()?;

    // Set the default branch for the remote
    Command::new("git")
        .args([
            "symbolic-ref",
            "HEAD",
            &format!("refs/heads/{remote_default}"),
        ])
        .current_dir(remote_path)
        .output()?;

    // Clone the bare repo to create our test repo
    Command::new("git")
        .args(["clone", remote_path.to_str().unwrap(), "."])
        .current_dir(repo_path)
        .output()?;

    // Configure git user for the test
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repository")?;
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    // Push to establish the remote branch
    Command::new("git")
        .args(["push", "-u", "origin", remote_default])
        .current_dir(repo_path)
        .output()?;

    // Set up origin/HEAD to point to the default branch
    Command::new("git")
        .args(["remote", "set-head", "origin", remote_default])
        .current_dir(repo_path)
        .output()?;

    // Keep the remote directory alive by leaking it
    // This is a test-only hack to prevent the remote from being deleted
    std::mem::forget(remote_dir);

    Ok(temp_dir)
}

#[test]
fn test_get_default_branch_with_symbolic_ref() -> Result<()> {
    let temp_repo = create_test_repo_with_remote_default("main")?;
    let repo_path = temp_repo.path();

    let default_branch = git::get_default_branch(repo_path)?;
    assert_eq!(default_branch, "origin/main");

    Ok(())
}

#[test]
fn test_get_default_branch_with_master() -> Result<()> {
    let temp_repo = create_test_repo_with_remote_default("master")?;
    let repo_path = temp_repo.path();

    let default_branch = git::get_default_branch(repo_path)?;
    assert_eq!(default_branch, "origin/master");

    Ok(())
}

#[test]
fn test_get_default_branch_with_custom_branch() -> Result<()> {
    let temp_repo = create_test_repo_with_remote_default("develop")?;
    let repo_path = temp_repo.path();

    let default_branch = git::get_default_branch(repo_path)?;
    assert_eq!(default_branch, "origin/develop");

    Ok(())
}

// Note: More complex fallback tests are omitted because they depend on git's internal behavior
// which can vary between versions and configurations. The tests above cover the main scenarios
// that matter for the rebase functionality.

#[test]
fn test_get_default_branch_prefers_main_over_master() -> Result<()> {
    // Test that when both main and master exist, main is preferred
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    // Create fake remote branches for both main and master
    let git_dir = repo_path.join(".git");
    let refs_remotes_dir = git_dir.join("refs").join("remotes").join("origin");
    fs::create_dir_all(&refs_remotes_dir)?;

    let fake_commit = "1234567890abcdef1234567890abcdef12345678";

    // Create both branches - main should be preferred
    fs::write(refs_remotes_dir.join("main"), format!("{fake_commit}\n"))?;
    fs::write(refs_remotes_dir.join("master"), format!("{fake_commit}\n"))?;

    let default_branch = git::get_default_branch(repo_path)?;
    assert_eq!(default_branch, "origin/main");

    Ok(())
}

// Note: branch_exists function testing is omitted due to complexity of git test setup
// The function is tested indirectly through the default branch detection tests above

#[test]
fn test_extract_account_from_source() -> Result<()> {
    use viewyard::git::extract_account_from_source;

    // Test personal repository
    assert_eq!(extract_account_from_source("GitHub (dheater)")?, "dheater");

    // Test organization repository
    assert_eq!(
        extract_account_from_source("GitHub (org/dheater)")?,
        "dheater"
    );

    // Test private repository
    assert_eq!(
        extract_account_from_source("GitHub (dheater) [private]")?,
        "dheater"
    );

    // Test organization private repository
    assert_eq!(
        extract_account_from_source("GitHub (myorg/dheater) [private]")?,
        "dheater"
    );

    // Test invalid formats
    assert!(extract_account_from_source("Not GitHub").is_err());
    assert!(extract_account_from_source("GitHub ()").is_err());
    assert!(extract_account_from_source("GitHub (org/)").is_err());

    Ok(())
}

#[test]
fn test_validate_repository_directory() -> Result<()> {
    use viewyard::git::validate_repository_directory;

    // Test with existing directory (current directory should exist)
    let current_dir = std::env::current_dir()?;
    assert!(validate_repository_directory(&current_dir, "current").is_ok());

    // Test with non-existent directory
    let non_existent = current_dir.join("this-directory-does-not-exist-12345");
    assert!(validate_repository_directory(&non_existent, "nonexistent").is_err());

    Ok(())
}

#[test]
fn test_git_config_operations() -> Result<()> {
    use viewyard::git::{get_git_config, set_git_config};

    let temp_repo = create_test_repo_with_remote_default("main")?;
    let repo_path = temp_repo.path();

    // Test setting and getting git config
    set_git_config("user.name", "testuser", repo_path)?;
    let retrieved_name = get_git_config("user.name", repo_path)?;
    assert_eq!(retrieved_name, "testuser");

    set_git_config("user.email", "test@example.com", repo_path)?;
    let retrieved_email = get_git_config("user.email", repo_path)?;
    assert_eq!(retrieved_email, "test@example.com");

    Ok(())
}

#[test]
fn test_validate_repository_for_operations() -> Result<()> {
    use viewyard::git::{get_git_config, validate_repository_for_operations};
    use viewyard::models::Repository;

    let temp_repo = create_test_repo_with_remote_default("main")?;
    let repo_path = temp_repo.path();

    // Create a test repository struct
    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/testuser/test-repo.git".to_string(),
        is_private: false,
        source: "GitHub (testuser)".to_string(),
        account: None,
    };

    // Test validation - should configure git user automatically
    validate_repository_for_operations(repo_path, &repo)?;

    // Verify git config was set correctly
    let configured_name = get_git_config("user.name", repo_path)?;
    let configured_email = get_git_config("user.email", repo_path)?;

    assert_eq!(configured_name, "testuser");
    assert_eq!(configured_email, "testuser@users.noreply.github.com");

    Ok(())
}

#[test]
fn test_validate_repository_with_explicit_account() -> Result<()> {
    use viewyard::git::{get_git_config, validate_repository_for_operations};
    use viewyard::models::Repository;

    let temp_repo = create_test_repo_with_remote_default("main")?;
    let repo_path = temp_repo.path();

    // Create a test repository struct with explicit account field
    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/org/test-repo.git".to_string(),
        is_private: false,
        source: "GitHub (org/someuser)".to_string(),
        account: Some("explicituser".to_string()),
    };

    // Test validation - should use explicit account, not source parsing
    validate_repository_for_operations(repo_path, &repo)?;

    // Verify git config was set to explicit account
    let configured_name = get_git_config("user.name", repo_path)?;
    let configured_email = get_git_config("user.email", repo_path)?;

    assert_eq!(configured_name, "explicituser");
    assert_eq!(configured_email, "explicituser@users.noreply.github.com");

    Ok(())
}

#[test]
fn test_signing_key_configuration() -> Result<()> {
    use viewyard::git::{get_git_config, set_git_config, validate_and_configure_git_user};

    let temp_repo = create_test_repo_with_remote_default("main")?;
    let repo_path = temp_repo.path();

    // SAFE: Set up a mock signing key in a separate test repository to simulate global config
    // This avoids modifying the user's actual global git configuration
    let mock_global_repo = TempDir::new()?;
    let mock_global_path = mock_global_repo.path();

    // Initialize a mock "global" repository for testing
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(mock_global_path)
        .output()?;

    // Set signing key in the mock repository (simulates global config)
    set_git_config("user.signingkey", "~/.ssh/test_key.pub", mock_global_path)?;

    // Test the signing key detection by temporarily overriding the function
    // For this test, we'll manually set the signing key in the target repo
    set_git_config("user.signingkey", "~/.ssh/test_key.pub", repo_path)?;

    // Test validation - should configure git user but preserve existing signing key
    validate_and_configure_git_user(repo_path, "testuser")?;

    // Verify git config was set correctly including signing key
    let configured_name = get_git_config("user.name", repo_path)?;
    let configured_email = get_git_config("user.email", repo_path)?;
    let configured_signing_key = get_git_config("user.signingkey", repo_path)?;

    assert_eq!(configured_name, "testuser");
    assert_eq!(configured_email, "testuser@users.noreply.github.com");
    assert_eq!(configured_signing_key, "~/.ssh/test_key.pub");

    Ok(())
}

#[test]
fn test_signing_key_detection() -> Result<()> {
    use viewyard::git::detect_signing_key;

    // SAFE: Test signing key detection without modifying global config
    // We test the function's behavior when no global signing key is configured
    // This is the safe default state and doesn't require global config modification

    // Test detection when no key is configured (safe default)
    let no_key = detect_signing_key();

    // This test verifies the function handles the "no signing key" case correctly
    // We cannot safely test the "signing key present" case without risking
    // modification of the user's global git configuration
    assert_eq!(no_key, None);

    Ok(())
}

#[test]
fn test_global_config_never_modified() -> Result<()> {
    use viewyard::git::{set_git_config, validate_and_configure_git_user};

    let temp_repo = create_test_repo_with_remote_default("main")?;
    let repo_path = temp_repo.path();

    // Capture initial global git config state (if any)
    let initial_global_name = std::process::Command::new("git")
        .args(["config", "--global", "user.name"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    let initial_global_email = std::process::Command::new("git")
        .args(["config", "--global", "user.email"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    let initial_global_signing_key = std::process::Command::new("git")
        .args(["config", "--global", "user.signingkey"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    // Perform viewyard operations that configure git
    validate_and_configure_git_user(repo_path, "testuser")?;
    set_git_config("user.name", "anotheruser", repo_path)?;
    set_git_config("user.email", "test@example.com", repo_path)?;

    // Verify global git config is unchanged
    let final_global_name = std::process::Command::new("git")
        .args(["config", "--global", "user.name"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    let final_global_email = std::process::Command::new("git")
        .args(["config", "--global", "user.email"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    let final_global_signing_key = std::process::Command::new("git")
        .args(["config", "--global", "user.signingkey"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    // Assert that global config is completely unchanged
    assert_eq!(initial_global_name, final_global_name,
        "Global git user.name was modified! This is a critical security violation.");
    assert_eq!(initial_global_email, final_global_email,
        "Global git user.email was modified! This is a critical security violation.");
    assert_eq!(initial_global_signing_key, final_global_signing_key,
        "Global git user.signingkey was modified! This is a critical security violation.");

    Ok(())
}
