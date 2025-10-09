use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Output};
use std::collections::HashMap;
use std::time::Duration;

// # Git Configuration Safety
//
// **CRITICAL SECURITY CONSTRAINT**: This module MUST NEVER modify global git configuration.
//
// ## Safety Rules:
// 1. All git config modifications MUST use `--local` flag only
// 2. Global config access is READ-ONLY via `GitConfigScope::GlobalReadOnly`
// 3. Tests MUST NOT modify global git configuration
// 4. The `set_git_config()` function is hardcoded to use `--local` only
//
// ## Rationale:
// Viewyard operates on multiple repositories and should never pollute the user's
// global git environment. All configuration should be repository-specific to
// maintain isolation and prevent unintended side effects.
//
// ## Enforcement:
// - Type system prevents global modifications via `GitConfigScope` enum
// - Tests verify global config is never modified
// - Code review must check for any `--global` usage

/// Run a git command and return the output
pub fn run_git_command(args: &[&str], working_dir: Option<&Path>) -> Result<Output> {
    run_git_command_with_timeout(args, working_dir, Duration::from_secs(30))
}

/// Run a git command with a timeout and return the output
pub fn run_git_command_with_timeout(
    args: &[&str],
    working_dir: Option<&Path>,
    _timeout: Duration,
) -> Result<Output> {
    let mut cmd = Command::new("git");
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // For now, we'll use the basic output() method
    // In a production system, you might want to implement proper timeout handling
    // using std::process::Child and thread-based timeouts
    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute git command: git {}", args.join(" ")))?;

    Ok(output)
}

/// Run a git command and return stdout as string
pub fn run_git_command_string(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let output = run_git_command(args, cwd)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: git {}\n{}", args.join(" "), stderr);
    }

    Ok(String::from_utf8(output.stdout)
        .context("Git command output is not valid UTF-8")?
        .trim()
        .to_string())
}

/// Run a git command and ensure it succeeds (helper for commands that don't need output)
/// Check if git is available on the system
pub fn check_git_availability() -> Result<()> {
    Command::new("git").args(["--version"]).output().context(
        "Git is not installed or not available in PATH. Please install git and try again.",
    )?;
    Ok(())
}

/// Check if a directory is a git repository
#[must_use]
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Get git status (porcelain format)
pub fn get_status(cwd: &Path) -> Result<String> {
    run_git_command_string(&["status", "--porcelain"], Some(cwd))
}

/// Get current branch name
pub fn get_current_branch(cwd: &Path) -> Result<String> {
    run_git_command_string(&["branch", "--show-current"], Some(cwd))
}

/// Check if repository has uncommitted changes
pub fn has_uncommitted_changes(cwd: &Path) -> Result<bool> {
    let status = get_status(cwd)?;
    Ok(!status.is_empty())
}

/// Check if repository has unpushed commits
pub fn has_unpushed_commits(cwd: &Path) -> Result<bool> {
    // Get commits ahead of origin
    let output = run_git_command(&["rev-list", "--count", "@{u}..HEAD"], Some(cwd));

    match output {
        Ok(output) => {
            if output.status.success() {
                let count_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let count: u32 = count_str
                    .parse()
                    .with_context(|| format!("Failed to parse commit count: '{count_str}'"))?;
                Ok(count > 0)
            } else {
                // Check git exit code for specific error conditions
                if output.status.code() == Some(128) {
                    // Exit code 128 typically means "no upstream configured"
                    Ok(false) // No upstream branch means no unpushed commits
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    anyhow::bail!("Failed to check for unpushed commits: {stderr}")
                }
            }
        }
        Err(e) => Err(e).context("Failed to execute git command to check unpushed commits"),
    }
}

/// Add all changes to staging
pub fn add_all(cwd: &Path) -> Result<()> {
    run_git_command(&["add", "."], Some(cwd))?;
    Ok(())
}

/// Commit changes with a message
pub fn commit(message: &str, cwd: &Path) -> Result<()> {
    run_git_command(&["commit", "-m", message], Some(cwd))?;
    Ok(())
}

/// Push to remote
pub fn push(cwd: &Path) -> Result<()> {
    run_git_command(&["push"], Some(cwd))?;
    Ok(())
}

/// Rebase against a branch
pub fn rebase(target_branch: &str, cwd: &Path) -> Result<()> {
    run_git_command(&["rebase", target_branch], Some(cwd))?;
    Ok(())
}

/// Fetch from remote
pub fn fetch(cwd: &Path) -> Result<()> {
    run_git_command(&["fetch"], Some(cwd))?;
    Ok(())
}

/// Get count of stashes
pub fn get_stash_count(cwd: &Path) -> Result<usize> {
    match run_git_command_string(&["stash", "list"], Some(cwd)) {
        Ok(output) => {
            if output.is_empty() {
                Ok(0)
            } else {
                Ok(output.lines().count())
            }
        }
        Err(e) => {
            // If git stash command fails, it's likely because the repo is not initialized
            // or there's a more serious git issue
            Err(e).context("Failed to get stash count")
        }
    }
}

/// Check if a branch exists
#[must_use]
pub fn branch_exists(branch_name: &str, cwd: &Path) -> bool {
    let result = run_git_command(
        &[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/remotes/{branch_name}"),
        ],
        Some(cwd),
    );
    result.is_ok()
}

/// Perform a fast-forward merge
pub fn merge_fast_forward(branch_name: &str, cwd: &Path) -> Result<()> {
    run_git_command(&["merge", "--ff-only", branch_name], Some(cwd))?;
    Ok(())
}

/// Get the default branch for the remote origin
pub fn get_default_branch(cwd: &Path) -> Result<String> {
    // Method 1: Try to get the symbolic ref for origin/HEAD
    if let Ok(output) =
        run_git_command_string(&["symbolic-ref", "refs/remotes/origin/HEAD"], Some(cwd))
    {
        // Output format: "refs/remotes/origin/main" -> extract "main"
        if let Some(branch_name) = output.strip_prefix("refs/remotes/origin/") {
            return Ok(format!("origin/{branch_name}"));
        }
    }

    // Method 2: Try to get default branch from remote show origin
    if let Ok(output) = run_git_command_string(&["remote", "show", "origin"], Some(cwd)) {
        for line in output.lines() {
            if let Some(branch) = line.strip_prefix("  HEAD branch: ") {
                return Ok(format!("origin/{}", branch.trim()));
            }
        }
    }

    // Method 3: Fall back to common defaults, checking which ones exist
    let common_defaults = ["origin/main", "origin/master", "origin/develop"];
    for &default in &common_defaults {
        if branch_exists(default, cwd) {
            return Ok(default.to_string());
        }
    }

    anyhow::bail!("Could not determine default branch for repository")
}

/// Detect SSH host aliases for GitHub from SSH config
/// Returns a map of account -> SSH host (e.g., "dheater" -> "github.com-dheater")
pub fn detect_ssh_host_aliases() -> HashMap<String, String> {
    let mut aliases = HashMap::new();

    // Try to read SSH config file
    let ssh_config_path = std::env::var("HOME")
        .map(|home| format!("{}/.ssh/config", home))
        .unwrap_or_else(|_| "/dev/null".to_string());

    if let Ok(config_content) = std::fs::read_to_string(&ssh_config_path) {
        let mut current_host: Option<String> = None;
        let mut current_hostname: Option<String> = None;

        for line in config_content.lines() {
            let line = line.trim();

            if line.starts_with("Host ") {
                // Process previous host if it was a GitHub alias
                if let (Some(host), Some(hostname)) = (&current_host, &current_hostname) {
                    if hostname == "github.com" && host.starts_with("github.com-") {
                        // Extract account from host alias (e.g., "github.com-dheater" -> "dheater")
                        if let Some(account) = host.strip_prefix("github.com-") {
                            aliases.insert(account.to_string(), host.clone());
                        }
                    }
                }

                // Start new host
                current_host = Some(line[5..].trim().to_string());
                current_hostname = None;
            } else if line.starts_with("HostName ") {
                current_hostname = Some(line[9..].trim().to_string());
            }
        }

        // Process the last host
        if let (Some(host), Some(hostname)) = (&current_host, &current_hostname) {
            if hostname == "github.com" && host.starts_with("github.com-") {
                if let Some(account) = host.strip_prefix("github.com-") {
                    aliases.insert(account.to_string(), host.clone());
                }
            }
        }
    }

    aliases
}

/// Transform a GitHub SSH URL to use the appropriate SSH host alias
/// Returns the original URL if no alias is found or if it's not a GitHub SSH URL
pub fn transform_github_url_for_account(url: &str, account: &str) -> String {
    // Only transform SSH URLs for github.com
    if !url.starts_with("git@github.com:") {
        return url.to_string();
    }

    let ssh_aliases = detect_ssh_host_aliases();

    if let Some(host_alias) = ssh_aliases.get(account) {
        // Replace "git@github.com:" with "git@{host_alias}:"
        url.replace("git@github.com:", &format!("git@{}:", host_alias))
    } else {
        // No SSH alias found, return original URL
        url.to_string()
    }
}

/// Extract GitHub account from repository source string
/// Supports formats: "GitHub (account)", "GitHub (org/account)", "GitHub (account) [private]"
///
/// # Panics
/// This function will not panic as it validates the source format before using `unwrap()`
pub fn extract_account_from_source(source: &str) -> Result<String> {
    if !source.contains("GitHub (") {
        anyhow::bail!("Source is not a GitHub repository: {}", source);
    }

    // Find the content between "GitHub (" and ")"
    let start = source.find("GitHub (").unwrap() + 8; // Length of "GitHub ("
    let remaining = &source[start..];

    if let Some(end) = remaining.find(')') {
        let account_part = &remaining[..end];

        // Handle organization repos: "org/account" -> extract "account"
        if let Some(slash_pos) = account_part.find('/') {
            let account = &account_part[slash_pos + 1..];
            if account.is_empty() {
                anyhow::bail!("Invalid account format in source: {}", source);
            }
            Ok(account.to_string())
        } else {
            // Personal repo: just the account name
            if account_part.is_empty() {
                anyhow::bail!("Invalid account format in source: {}", source);
            }
            Ok(account_part.to_string())
        }
    } else {
        anyhow::bail!("Malformed source format: {}", source);
    }
}

/// Safe git configuration scope - prevents global modifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitConfigScope {
    /// Repository-local configuration only (safe)
    Local,
    /// Read-only access to global configuration (safe for reading)
    GlobalReadOnly,
}

/// Get git configuration value for a specific key with explicit scope
pub fn get_git_config_scoped(key: &str, scope: GitConfigScope, cwd: Option<&Path>) -> Result<String> {
    match scope {
        GitConfigScope::Local => {
            let repo_path = cwd.ok_or_else(|| anyhow::anyhow!("Repository path required for local config"))?;
            run_git_command_string(&["config", "--local", key], Some(repo_path))
        }
        GitConfigScope::GlobalReadOnly => {
            run_git_command_string(&["config", "--global", key], None)
        }
    }
}

/// Set git configuration value for a specific key (LOCAL ONLY - safe)
/// This function ONLY allows local repository configuration to prevent
/// accidental modification of global git settings
pub fn set_git_config(key: &str, value: &str, cwd: &Path) -> Result<()> {
    // SAFETY: This function is hardcoded to only use --local flag
    // to prevent any possibility of modifying global git configuration
    run_git_command(&["config", "--local", key, value], Some(cwd))?;
    Ok(())
}

/// Get git configuration value for a specific key in a repository (LOCAL ONLY)
pub fn get_git_config(key: &str, cwd: &Path) -> Result<String> {
    get_git_config_scoped(key, GitConfigScope::Local, Some(cwd))
}

/// Get git configuration value from global config (READ-ONLY)
/// This function is explicitly marked as read-only to emphasize that
/// viewyard NEVER modifies global git configuration
pub fn get_global_git_config(key: &str) -> Result<String> {
    get_git_config_scoped(key, GitConfigScope::GlobalReadOnly, None)
}

/// Detect available signing key from global git configuration
#[must_use]
pub fn detect_signing_key() -> Option<String> {
    // Try to get signing key from global config
    if let Ok(signing_key) = get_global_git_config("user.signingkey") {
        if !signing_key.trim().is_empty() {
            return Some(signing_key.trim().to_string());
        }
    }
    None
}

/// Validate and configure git user settings for a repository
pub fn validate_and_configure_git_user(repo_path: &Path, account: &str) -> Result<()> {
    // Check current configuration
    let current_name = get_git_config("user.name", repo_path).ok();
    let current_email = get_git_config("user.email", repo_path).ok();
    let current_signing_key = get_git_config("user.signingkey", repo_path).ok();

    let expected_email = format!("{account}@users.noreply.github.com");

    // Configure user.name if not set or incorrect
    let name_configured = if current_name.as_deref() == Some(account) {
        false
    } else {
        set_git_config("user.name", account, repo_path)
            .with_context(|| format!("Failed to set user.name to '{account}'"))?;
        true
    };

    // Configure user.email if not set or incorrect
    let email_configured = if current_email.as_deref() == Some(&expected_email) {
        false
    } else {
        set_git_config("user.email", &expected_email, repo_path)
            .with_context(|| format!("Failed to set user.email to '{expected_email}'"))?;
        true
    };

    // Configure signing key if available and not already set
    let signing_key_configured = if let Some(global_signing_key) = detect_signing_key() {
        if current_signing_key.as_deref() == Some(&global_signing_key) {
            false
        } else {
            set_git_config("user.signingkey", &global_signing_key, repo_path).with_context(
                || format!("Failed to set user.signingkey to '{global_signing_key}'"),
            )?;
            true
        }
    } else {
        false
    };

    // Provide feedback about what was configured
    if name_configured || email_configured || signing_key_configured {
        use crate::ui;
        let mut config_parts = vec![format!("{account} <{expected_email}>")];

        if signing_key_configured {
            if let Some(signing_key) = detect_signing_key() {
                // Show a shortened version of the signing key for readability
                let key_display = if signing_key.len() > 20 {
                    format!("{}...", &signing_key[..20])
                } else {
                    signing_key
                };
                config_parts.push(format!("signing: {key_display}"));
            }
        }

        ui::print_info(&format!(
            "Configured git user: {}",
            config_parts.join(", ")
        ));
    }

    Ok(())
}

/// Comprehensive validation for git repository and user configuration
/// This function should be called before any git operations in workspace commands
pub fn validate_repository_for_operations(
    repo_path: &Path,
    repo: &crate::models::Repository,
) -> Result<()> {
    // 1. Verify this is actually a git repository
    if !is_git_repo(repo_path) {
        anyhow::bail!(
            "Directory '{}' is not a git repository (missing .git directory)",
            repo.name
        );
    }

    // 2. Determine account - prefer explicit account field, fall back to source parsing
    let account = if let Some(ref explicit_account) = repo.account {
        explicit_account.clone()
    } else {
        extract_account_from_source(&repo.source).with_context(|| {
            format!(
                "Failed to extract GitHub account from source: {}",
                repo.source
            )
        })?
    };

    // 3. Validate and configure git user settings
    validate_and_configure_git_user(repo_path, &account)
        .with_context(|| format!("Failed to configure git user for repository: {}", repo.name))?;

    Ok(())
}

/// Validate that a directory exists and is accessible
pub fn validate_repository_directory(repo_path: &Path, repo_name: &str) -> Result<()> {
    if !repo_path.exists() {
        anyhow::bail!(
            "Repository directory '{}' does not exist: {}",
            repo_name,
            repo_path.display()
        );
    }

    if !repo_path.is_dir() {
        anyhow::bail!(
            "Repository path '{}' is not a directory: {}",
            repo_name,
            repo_path.display()
        );
    }

    Ok(())
}
