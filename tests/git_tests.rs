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
