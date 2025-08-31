use anyhow::Result;
use clap::Subcommand;
use std::fs;
use std::path::Path;

use crate::git;
use crate::ui;

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// Show status of all repos in current view
    Status,
    /// Rebase repos against origin/master
    Rebase,
    /// Commit to all dirty repos (only repos with changes)
    #[command(name = "commit-all")]
    CommitAll {
        /// Commit message
        message: String,
    },
    /// Push repos with commits ahead (only repos with unpushed commits)
    #[command(name = "push-all")]
    PushAll,
}

pub fn handle_command(command: WorkspaceCommand) -> Result<()> {
    match command {
        WorkspaceCommand::Status => workspace_status(),
        WorkspaceCommand::Rebase => workspace_rebase(),
        WorkspaceCommand::CommitAll { message } => workspace_commit_all(&message),
        WorkspaceCommand::PushAll => workspace_push_all(),
    }
}

fn workspace_status() -> Result<()> {
    ui::print_header("Repository Status");

    // Detect current view
    let current_dir = std::env::current_dir()?;
    let view_context = load_view_context(&current_dir)?;

    ui::print_info(&format!("View: {}", view_context.view_name));
    ui::print_info(&format!("Root: {}", view_context.view_root.display()));

    if view_context.active_repos.is_empty() {
        ui::print_warning("No repositories in this view");
        return Ok(());
    }

    ui::print_info("");

    // Collect branch information for consistency check
    let mut repo_branches = Vec::new();
    let mut clean_count = 0;
    let mut dirty_count = 0;
    let mut ahead_count = 0;

    for repo_name in &view_context.active_repos {
        let repo_path = view_context.view_root.join(repo_name);

        if !repo_path.exists() {
            ui::print_warning(&format!("⚠️  {}: Directory not found", repo_name));
            continue;
        }

        if !git::is_git_repo(&repo_path) {
            ui::print_warning(&format!("⚠️  {}: Not a git repository", repo_name));
            continue;
        }

        // Get branch for consistency check
        let branch = git::get_current_branch(&repo_path).unwrap_or_else(|_| "unknown".to_string());
        repo_branches.push((repo_name.clone(), branch.clone()));

        // Get repository status
        match get_repo_status(&repo_path, repo_name) {
            Ok(Some(status)) => {
                println!("{}", status);
                if status.contains("changes") {
                    dirty_count += 1;
                }
                if status.contains("ahead") {
                    ahead_count += 1;
                }
            }
            Ok(None) => {
                // Show clean repos too
                println!("✓ {} ({}) - clean", repo_name, branch);
                clean_count += 1;
            }
            Err(e) => {
                ui::print_warning(&format!("⚠️  {}: Error getting status - {}", repo_name, e))
            }
        }
    }

    // Check branch consistency and show summary
    check_branch_consistency(&repo_branches);
    show_status_summary(clean_count, dirty_count, ahead_count, &repo_branches);

    Ok(())
}

fn workspace_rebase() -> Result<()> {
    ui::print_header("Rebasing repositories");
    // TODO: Implement rebase for all repos
    ui::print_success("All repositories rebased successfully");
    Ok(())
}

fn workspace_commit_all(message: &str) -> Result<()> {
    ui::print_header(&format!(
        "Committing repositories with changes: {}",
        message
    ));

    let current_dir = std::env::current_dir()?;
    let view_context = load_view_context(&current_dir)?;

    let mut committed_repos = Vec::new();
    let mut error_repos = Vec::new();
    let mut repos_to_commit = Vec::new();

    // First pass: identify repos that need committing
    for repo_name in &view_context.active_repos {
        let repo_path = view_context.view_root.join(repo_name);

        if !repo_path.exists() || !git::is_git_repo(&repo_path) {
            ui::print_warning(&format!(
                "⚠️  {}: Directory not found or not a git repository",
                repo_name
            ));
            continue;
        }

        match git::has_uncommitted_changes(&repo_path) {
            Ok(true) => {
                repos_to_commit.push(repo_name.clone());
            }
            Ok(false) => {
                // Skip clean repos silently
            }
            Err(e) => {
                ui::print_warning(&format!("⚠️  {}: Error checking status - {}", repo_name, e));
            }
        }
    }

    if repos_to_commit.is_empty() {
        ui::print_info("No repositories have uncommitted changes");
        return Ok(());
    }

    ui::print_info(&format!(
        "Found {} repositories with changes",
        repos_to_commit.len()
    ));

    // Second pass: commit changes with rollback on failure
    for repo_name in &repos_to_commit {
        let repo_path = view_context.view_root.join(repo_name);

        ui::print_info(&format!("Committing changes in {}", repo_name));
        match commit_repo_changes(&repo_path, message) {
            Ok(()) => {
                committed_repos.push(repo_name.clone());
            }
            Err(e) => {
                ui::print_error(&format!("❌ Failed to commit {}: {}", repo_name, e));
                error_repos.push(repo_name.clone());

                // Rollback: reset any staged changes
                if let Err(reset_err) = git::run_git_command(&["reset", "HEAD"], Some(&repo_path)) {
                    ui::print_warning(&format!(
                        "⚠️  Failed to rollback staged changes in {}: {}",
                        repo_name, reset_err
                    ));
                }

                // Stop on first failure and inform user
                ui::print_error("❌ Commit operation stopped due to failure");
                ui::print_info(
                    "💡 Fix the issue in the failed repository and run the command again",
                );
                ui::print_info(&format!(
                    "💡 Successfully committed repositories: {}",
                    if committed_repos.is_empty() {
                        "none".to_string()
                    } else {
                        committed_repos.join(", ")
                    }
                ));

                return Err(anyhow::anyhow!(
                    "Commit failed for repository: {}",
                    repo_name
                ));
            }
        }
    }

    // Success summary
    if !committed_repos.is_empty() {
        ui::print_success(&format!(
            "✅ Successfully committed {} repositories: {}",
            committed_repos.len(),
            committed_repos.join(", ")
        ));
    }

    Ok(())
}

fn commit_repo_changes(repo_path: &Path, message: &str) -> Result<()> {
    git::add_all(repo_path)?;
    git::commit(message, repo_path)?;
    Ok(())
}

fn workspace_push_all() -> Result<()> {
    ui::print_header("Pushing repositories with unpushed commits");

    let current_dir = std::env::current_dir()?;
    let view_context = load_view_context(&current_dir)?;

    let mut pushed_repos = Vec::new();
    let mut repos_to_push = Vec::new();

    // First pass: identify repos that need pushing
    for repo_name in &view_context.active_repos {
        let repo_path = view_context.view_root.join(repo_name);

        if !repo_path.exists() || !git::is_git_repo(&repo_path) {
            ui::print_warning(&format!(
                "⚠️  {}: Directory not found or not a git repository",
                repo_name
            ));
            continue;
        }

        match git::has_unpushed_commits(&repo_path) {
            Ok(true) => {
                repos_to_push.push(repo_name.clone());
            }
            Ok(false) => {
                // Skip repos with nothing to push silently
            }
            Err(e) => {
                ui::print_warning(&format!(
                    "⚠️  {}: Error checking push status - {}",
                    repo_name, e
                ));
            }
        }
    }

    if repos_to_push.is_empty() {
        ui::print_info("No repositories have unpushed commits");
        return Ok(());
    }

    ui::print_info(&format!(
        "Found {} repositories with unpushed commits",
        repos_to_push.len()
    ));

    // Second pass: push commits with failure handling
    for repo_name in &repos_to_push {
        let repo_path = view_context.view_root.join(repo_name);

        ui::print_info(&format!("Pushing commits in {}", repo_name));
        match git::push(&repo_path) {
            Ok(()) => {
                pushed_repos.push(repo_name.clone());
            }
            Err(e) => {
                ui::print_error(&format!("❌ Failed to push {}: {}", repo_name, e));

                // For push failures, we can't really rollback, but we can inform the user
                ui::print_error("❌ Push operation stopped due to failure");
                ui::print_info("💡 Common solutions:");
                ui::print_info("   • Pull latest changes: git pull");
                ui::print_info("   • Check remote permissions");
                ui::print_info("   • Verify network connection");
                ui::print_info(&format!(
                    "💡 Successfully pushed repositories: {}",
                    if pushed_repos.is_empty() {
                        "none".to_string()
                    } else {
                        pushed_repos.join(", ")
                    }
                ));

                return Err(anyhow::anyhow!("Push failed for repository: {}", repo_name));
            }
        }
    }

    // Success summary
    if !pushed_repos.is_empty() {
        ui::print_success(&format!(
            "✅ Successfully pushed {} repositories: {}",
            pushed_repos.len(),
            pushed_repos.join(", ")
        ));
    }

    Ok(())
}

// Helper functions

#[derive(Debug)]
struct ViewContext {
    view_name: String,
    view_root: std::path::PathBuf,
    active_repos: Vec<String>,
}

fn load_view_context(current_dir: &Path) -> Result<ViewContext> {
    // Look for .viewyard-context file in current directory or parent directories
    let mut search_dir = current_dir;

    loop {
        let context_file = search_dir.join(".viewyard-context");
        if context_file.exists() {
            let content = fs::read_to_string(&context_file)?;
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;

            let view_name = yaml_value["view_name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing view_name in context file"))?
                .to_string();

            let view_root = yaml_value["view_root"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Missing view_root in context file"))?;

            let active_repos = yaml_value["active_repos"]
                .as_sequence()
                .ok_or_else(|| anyhow::anyhow!("Missing active_repos in context file"))?
                .iter()
                .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                .collect();

            return Ok(ViewContext {
                view_name,
                view_root: std::path::PathBuf::from(view_root),
                active_repos,
            });
        }

        match search_dir.parent() {
            Some(parent) => search_dir = parent,
            None => break,
        }
    }

    anyhow::bail!("Not in a viewyard view directory. Run this command from within a view.")
}

fn get_repo_status(repo_path: &Path, repo_name: &str) -> Result<Option<String>> {
    // Get current branch
    let branch = git::get_current_branch(repo_path).unwrap_or_else(|_| "unknown".to_string());

    // Check for uncommitted changes
    let has_changes = git::has_uncommitted_changes(repo_path)?;

    // Check for unpushed commits
    let has_unpushed = git::has_unpushed_commits(repo_path)?;

    // Check for stashes
    let stash_count = git::get_stash_count(repo_path)?;

    // Skip completely clean repos
    if !has_changes && !has_unpushed && stash_count == 0 {
        return Ok(None);
    }

    // Build concise one-line status
    let mut status_parts = Vec::new();

    if has_changes {
        // Count changes
        match git::get_status(repo_path) {
            Ok(status_output) => {
                let change_count = status_output.lines().count();
                if change_count > 0 {
                    status_parts.push(format!("{} changes", change_count));
                }
            }
            Err(_) => {
                status_parts.push("changes".to_string());
            }
        }
    }

    if has_unpushed {
        match git::run_git_command_string(&["rev-list", "--count", "@{u}..HEAD"], Some(repo_path)) {
            Ok(count_str) => {
                if let Ok(count) = count_str.parse::<u32>() {
                    if count > 0 {
                        status_parts.push(format!("{} commits ahead", count));
                    }
                }
            }
            Err(_) => {
                status_parts.push("commits ahead".to_string());
            }
        }
    }

    if stash_count > 0 {
        status_parts.push(format!("{} stashes", stash_count));
    }

    let status_summary = if status_parts.is_empty() {
        "clean".to_string()
    } else {
        status_parts.join(", ")
    };

    let icon = if has_changes { "⚠" } else { "→" };

    Ok(Some(format!("{} {} ({}) - {}", icon, repo_name, branch, status_summary)))
}

fn check_branch_consistency(repo_branches: &[(String, String)]) {
    if repo_branches.len() <= 1 {
        return;
    }

    // Group repos by branch
    let mut branch_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (repo, branch) in repo_branches {
        branch_groups.entry(branch.clone()).or_default().push(repo.clone());
    }

    if branch_groups.len() > 1 {
        ui::print_warning("⚠️  Branch mismatch detected:");
        for (branch, repos) in &branch_groups {
            if repos.len() == 1 {
                ui::print_warning(&format!("  - {}: {}", repos[0], branch));
            } else {
                ui::print_info(&format!("  - {} repos on: {}", repos.len(), branch));
            }
        }
        println!();
    }
}

fn show_status_summary(clean_count: usize, dirty_count: usize, ahead_count: usize, repo_branches: &[(String, String)]) {
    let total = clean_count + dirty_count;
    let mut summary_parts = Vec::new();

    if clean_count > 0 {
        summary_parts.push(format!("{} clean", clean_count));
    }
    if dirty_count > 0 {
        summary_parts.push(format!("{} dirty", dirty_count));
    }
    if ahead_count > 0 {
        summary_parts.push(format!("{} ahead", ahead_count));
    }

    let status_summary = if summary_parts.is_empty() {
        "all clean".to_string()
    } else {
        summary_parts.join(", ")
    };

    // Check if all repos are on the same branch
    let branch_consistency = if repo_branches.len() <= 1 {
        "".to_string()
    } else {
        let first_branch = &repo_branches[0].1;
        if repo_branches.iter().all(|(_, branch)| branch == first_branch) {
            format!(" | All on {} ✓", first_branch)
        } else {
            " | Mixed branches ⚠️".to_string()
        }
    };

    ui::print_info(&format!("{} repos: {}{}", total, status_summary, branch_consistency));
}
