use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Output};
use std::time::Duration;

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
