use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Output};

/// Run a git command and return the output
pub fn run_git_command(args: &[&str], working_dir: Option<&Path>) -> Result<Output> {
    let mut cmd = Command::new("git");
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

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

/// Check if a directory is a git repository
#[must_use]
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Initialize a new git repository
pub fn init_repo(path: &Path) -> Result<()> {
    run_git_command(&["init"], Some(path))?;
    Ok(())
}

/// Clone a repository
pub fn clone_repo(url: &str, target_dir: &Path) -> Result<()> {
    let parent = target_dir
        .parent()
        .context("Target directory has no parent")?;

    let dir_name = target_dir
        .file_name()
        .context("Target directory has no name")?
        .to_string_lossy();

    run_git_command(&["clone", url, &dir_name], Some(parent))?;
    Ok(())
}

/// Add a submodule to a repository
pub fn add_submodule(repo_url: &str, path: &str, cwd: &Path) -> Result<()> {
    run_git_command(&["submodule", "add", repo_url, path], Some(cwd))?;
    Ok(())
}

/// Update submodules
pub fn update_submodules(cwd: &Path) -> Result<()> {
    run_git_command(&["submodule", "update", "--init", "--recursive"], Some(cwd))?;
    Ok(())
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
    let result = run_git_command_string(&["rev-list", "--count", "@{u}..HEAD"], Some(cwd));

    result.map_or_else(|_| Ok(false), |count_str| {
        let count: u32 = count_str.parse().unwrap_or(0);
        Ok(count > 0)
    })
}

/// Create and checkout a new branch
pub fn create_branch(branch_name: &str, cwd: &Path) -> Result<()> {
    run_git_command(&["checkout", "-b", branch_name], Some(cwd))?;
    Ok(())
}

/// Checkout an existing branch
pub fn checkout_branch(branch_name: &str, cwd: &Path) -> Result<()> {
    run_git_command(&["checkout", branch_name], Some(cwd))?;
    Ok(())
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

/// Push with upstream tracking
pub fn push_set_upstream(branch_name: &str, remote: &str, cwd: &Path) -> Result<()> {
    run_git_command(&["push", "-u", remote, branch_name], Some(cwd))?;
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

/// Set git config value
pub fn set_config(key: &str, value: &str, global: bool) -> Result<()> {
    let mut args = vec!["config"];
    if global {
        args.push("--global");
    }
    args.extend(&[key, value]);

    run_git_command(&args, None)?;
    Ok(())
}

/// Get git config value
pub fn get_config(key: &str, global: bool) -> Result<String> {
    let mut args = vec!["config"];
    if global {
        args.push("--global");
    }
    args.push(key);

    run_git_command_string(&args, None)
}

/// Get detailed git status (human readable format)
pub fn get_detailed_status(cwd: &Path) -> Result<String> {
    run_git_command_string(&["status"], Some(cwd))
}

/// Get count of stashes
pub fn get_stash_count(cwd: &Path) -> Result<usize> {
    let result = run_git_command_string(&["stash", "list"], Some(cwd));

    result.map_or_else(|_| Ok(0), |output| if output.is_empty() {
        Ok(0)
    } else {
        Ok(output.lines().count())
    })
}

/// Get information about unpushed commits
pub fn get_unpushed_commits_info(cwd: &Path) -> Result<String> {
    // Get count of commits ahead
    let count_result = run_git_command_string(&["rev-list", "--count", "@{u}..HEAD"], Some(cwd));

    count_result.map_or_else(|_| Ok("unknown".to_string()), |count_str| {
        let count: usize = count_str.parse().unwrap_or(0);
        if count == 0 {
            Ok("0".to_string())
        } else if count == 1 {
            Ok("1 commit".to_string())
        } else {
            Ok(format!("{count} commits"))
        }
    })
}
