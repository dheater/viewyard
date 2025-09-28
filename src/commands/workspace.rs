use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::Path;

use crate::git;
use crate::models;
use crate::ui;

/// Validate and load repository configuration from JSON file
fn load_and_validate_repos(repos_file: &Path) -> Result<Vec<models::Repository>> {
    let repos_json = std::fs::read_to_string(repos_file).with_context(|| {
        format!(
            "Failed to read configuration file: {}",
            repos_file.display()
        )
    })?;

    let repositories: Vec<models::Repository> = serde_json::from_str(&repos_json)
        .with_context(|| {
            format!(
                "Invalid JSON in configuration file: {}\n\
                Expected format: array of repository objects with 'name', 'url', 'is_private', and 'source' fields",
                repos_file.display()
            )
        })?;

    // Validate each repository entry
    for (index, repo) in repositories.iter().enumerate() {
        if repo.name.trim().is_empty() {
            anyhow::bail!(
                "Invalid repository at index {}: 'name' field cannot be empty\n\
                File: {}",
                index,
                repos_file.display()
            );
        }

        if repo.url.trim().is_empty() {
            anyhow::bail!(
                "Invalid repository at index {}: 'url' field cannot be empty\n\
                Repository: {}\n\
                File: {}",
                index,
                repo.name,
                repos_file.display()
            );
        }

        // Basic URL validation - should contain git-like patterns
        if !repo.url.contains("git") && !repo.url.contains("github") && !repo.url.contains("gitlab")
        {
            ui::print_warning(&format!(
                "Repository '{}' has unusual URL format: {}\n\
                This might not be a valid Git repository URL",
                repo.name, repo.url
            ));
        }
    }

    Ok(repositories)
}

fn repo_directory_name(repo: &models::Repository) -> &str {
    repo.directory_name.as_deref().unwrap_or(&repo.name)
}

fn resolve_repo_path(view_root: &Path, repo: &models::Repository) -> std::path::PathBuf {
    view_root.join(repo_directory_name(repo))
}

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// Show status of all repos in current view
    Status,
    /// Rebase repos against their default branch
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

/// Handle workspace commands that operate on all repositories in the current view
///
/// These commands must be run from within a view directory and will validate
/// that all repositories are synchronized on the same branch before proceeding.
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
    let view_context =
        load_view_context(&current_dir).with_context(|| "Failed to run 'viewyard status'")?;

    ui::print_info(&format!("Viewset: {}", view_context.viewset_name));
    ui::print_info(&format!("View: {}", view_context.view_name));
    ui::print_info(&format!("Root: {}", view_context.view_root.display()));

    if view_context.active_repos.is_empty() {
        ui::print_warning("No repositories in this view");
        return Ok(());
    }

    // Validate branch synchronization
    if let Err(e) = validate_branch_synchronization(&view_context) {
        ui::print_warning(&format!("Branch synchronization check failed: {e}"));
        ui::print_info("Continuing with status check...");
    }

    ui::print_info("");

    // Collect branch information for consistency check
    let mut repo_branches = Vec::new();
    let mut clean_count = 0;
    let mut dirty_count = 0;
    let mut ahead_count = 0;

    for repo in &view_context.active_repos {
        let repo_path = resolve_repo_path(&view_context.view_root, repo);
        let repo_dir_name = repo_directory_name(repo);

        // Validate directory exists
        if let Err(e) = git::validate_repository_directory(&repo_path, repo_dir_name) {
            ui::print_warning(&format!("{}: {}", repo.name, e));
            continue;
        }

        // Validate git repository and user configuration (but don't fail on config issues for status)
        if let Err(e) = git::validate_repository_for_operations(&repo_path, repo) {
            ui::print_warning(&format!("{}: Git configuration issue - {}", repo.name, e));
            // Continue with status check even if git config has issues
        }

        // Get branch for consistency check
        let branch = git::get_current_branch(&repo_path).unwrap_or_else(|_| "unknown".to_string());
        repo_branches.push((repo.name.clone(), branch.clone()));

        // Get repository status
        match get_repo_status(&repo_path, &repo.name) {
            Ok(Some(status)) => {
                println!("{status}");
                if status.contains("changes") {
                    dirty_count += 1;
                }
                if status.contains("ahead") {
                    ahead_count += 1;
                }
            }
            Ok(None) => {
                // Show clean repos too
                println!("✓ {} ({}) - clean", repo.name, branch);
                clean_count += 1;
            }
            Err(e) => {
                ui::print_warning(&format!("{}: Error getting status - {}", repo.name, e));
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

    let current_dir = std::env::current_dir()?;
    let view_context =
        load_view_context(&current_dir).with_context(|| "Failed to run 'viewyard rebase'")?;

    let mut rebased_repos = Vec::new();
    let mut error_repos = Vec::new();
    let mut repos_to_rebase: Vec<models::Repository> = Vec::new();

    // First pass: validate repositories and git configuration
    for repo in &view_context.active_repos {
        let repo_path = resolve_repo_path(&view_context.view_root, repo);
        let repo_dir_name = repo_directory_name(repo);

        // Validate directory exists
        if let Err(e) = git::validate_repository_directory(&repo_path, repo_dir_name) {
            ui::print_warning(&format!("{}: {}", repo.name, e));
            continue;
        }

        // Validate git repository and user configuration
        if let Err(e) = git::validate_repository_for_operations(&repo_path, repo) {
            ui::print_warning(&format!("{}: {}", repo.name, e));
            continue;
        }

        repos_to_rebase.push(repo.clone());
    }

    if repos_to_rebase.is_empty() {
        ui::print_info("No repositories found to rebase");
        return Ok(());
    }

    ui::print_info(&format!(
        "Found {} repositories to rebase",
        repos_to_rebase.len()
    ));

    // Second pass: perform rebase operations
    for repo in repos_to_rebase {
        let repo_name = repo.name.clone();
        let repo_path = resolve_repo_path(&view_context.view_root, &repo);

        ui::print_info(&format!("Rebasing {repo_name}"));

        match rebase_repo(&repo_path) {
            Ok(()) => {
                ui::print_success(&format!("{repo_name}: Rebased successfully"));
                rebased_repos.push(repo_name);
            }
            Err(e) => {
                ui::print_error(&format!("{repo_name}: Failed to rebase - {e}"));
                error_repos.push((repo_name, e.to_string()));
            }
        }
    }

    // Summary
    if !rebased_repos.is_empty() {
        ui::print_success(&format!(
            "Successfully rebased {} repositories: {}",
            rebased_repos.len(),
            rebased_repos.join(", ")
        ));
    }

    if !error_repos.is_empty() {
        ui::print_error(&format!(
            "Failed to rebase {} repositories",
            error_repos.len()
        ));
        for (repo, error) in &error_repos {
            ui::print_error(&format!("   {repo}: {error}"));
        }
        anyhow::bail!("Some repositories failed to rebase");
    }

    Ok(())
}

fn workspace_commit_all(message: &str) -> Result<()> {
    ui::print_header(&format!("Committing repositories with changes: {message}"));

    let current_dir = std::env::current_dir()?;
    let view_context =
        load_view_context(&current_dir).with_context(|| "Failed to run 'viewyard commit-all'")?;

    let mut committed_repos = Vec::new();
    let mut repos_to_commit: Vec<models::Repository> = Vec::new();

    // First pass: validate repositories and identify repos that need committing
    for repo in &view_context.active_repos {
        let repo_path = resolve_repo_path(&view_context.view_root, repo);
        let repo_dir_name = repo_directory_name(repo);

        // Validate directory exists
        if let Err(e) = git::validate_repository_directory(&repo_path, repo_dir_name) {
            ui::print_warning(&format!("{}: {}", repo.name, e));
            continue;
        }

        // Validate git repository and user configuration
        if let Err(e) = git::validate_repository_for_operations(&repo_path, repo) {
            ui::print_warning(&format!("{}: {}", repo.name, e));
            continue;
        }

        match git::has_uncommitted_changes(&repo_path) {
            Ok(true) => {
                repos_to_commit.push(repo.clone());
            }
            Ok(false) => {
                // Skip clean repos silently
            }
            Err(e) => {
                ui::print_warning(&format!("{}: Error checking status - {}", repo.name, e));
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
    for repo in &repos_to_commit {
        let repo_name = &repo.name;
        let repo_path = resolve_repo_path(&view_context.view_root, repo);

        ui::print_info(&format!("Committing changes in {repo_name}"));
        match commit_repo_changes(&repo_path, message) {
            Ok(()) => {
                committed_repos.push(repo_name.clone());
            }
            Err(e) => {
                ui::print_error(&format!("Failed to commit {repo_name}: {e}"));

                // Rollback: reset any staged changes
                if let Err(reset_err) = git::run_git_command(&["reset", "HEAD"], Some(&repo_path)) {
                    ui::print_warning(&format!(
                        "Failed to rollback staged changes in {repo_name}: {reset_err}"
                    ));
                }

                // Stop on first failure and inform user
                ui::print_error("Commit operation stopped due to failure");
                ui::print_info("Fix the issue in the failed repository and run the command again");
                ui::print_info(&format!(
                    "Successfully committed repositories: {}",
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
            "Successfully committed {} repositories: {}",
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

fn rebase_repo(repo_path: &Path) -> Result<()> {
    // Check for clean working directory first
    if git::has_uncommitted_changes(repo_path)? {
        anyhow::bail!(
            "Cannot rebase with uncommitted changes. Please commit or stash your changes first."
        );
    }

    // First, fetch the latest changes
    git::fetch(repo_path)?;

    // Get the current branch name
    let current_branch = git::get_current_branch(repo_path)?;

    // Dynamically detect the default branch for this repository
    let rebase_target = git::get_default_branch(repo_path)
        .with_context(|| "Failed to detect default branch for repository")?;

    // Extract the branch name from the rebase target (e.g., "origin/main" -> "main")
    let target_branch_name = rebase_target
        .strip_prefix("origin/")
        .unwrap_or(&rebase_target);

    // Check if we're already on the target branch
    if current_branch == target_branch_name {
        // If we're on the default branch, just fast-forward merge
        git::merge_fast_forward(&rebase_target, repo_path)?;
        Ok(())
    } else {
        // Attempt rebase with conflict detection
        match git::rebase(&rebase_target, repo_path) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Check if we're in a rebase state (conflict occurred)
                if repo_path.join(".git/rebase-merge").exists()
                    || repo_path.join(".git/rebase-apply").exists()
                {
                    ui::print_error("Rebase conflict detected!");
                    ui::print_info("Manual resolution required:");
                    ui::print_info("   1. Navigate to the repository:");
                    ui::print_info(&format!("      cd {}", repo_path.display()));
                    ui::print_info("   2. Resolve conflicts in the affected files");
                    ui::print_info("   3. Stage resolved files: git add <file>");
                    ui::print_info("   4. Continue rebase: git rebase --continue");
                    ui::print_info("   5. Or abort rebase: git rebase --abort");
                    ui::print_info("");
                    ui::print_info("Common conflict resolution:");
                    ui::print_info("   • Edit files to resolve <<<< ==== >>>> markers");
                    ui::print_info("   • Use 'git status' to see conflicted files");
                    ui::print_info("   • Use 'git diff' to see conflict details");

                    anyhow::bail!("Rebase conflict requires manual resolution")
                }
                // Some other rebase error
                Err(e).context("Rebase failed")
            }
        }
    }
}

fn workspace_push_all() -> Result<()> {
    ui::print_header("Pushing repositories with unpushed commits");

    let current_dir = std::env::current_dir()?;
    let view_context =
        load_view_context(&current_dir).with_context(|| "Failed to run 'viewyard push-all'")?;

    let mut pushed_repos = Vec::new();
    let mut repos_to_push: Vec<models::Repository> = Vec::new();

    // First pass: validate repositories and identify repos that need pushing
    for repo in &view_context.active_repos {
        let repo_path = resolve_repo_path(&view_context.view_root, repo);
        let repo_dir_name = repo_directory_name(repo);

        // Validate directory exists
        if let Err(e) = git::validate_repository_directory(&repo_path, repo_dir_name) {
            ui::print_warning(&format!("{}: {}", repo.name, e));
            continue;
        }

        // Validate git repository and user configuration
        if let Err(e) = git::validate_repository_for_operations(&repo_path, repo) {
            ui::print_warning(&format!("{}: {}", repo.name, e));
            continue;
        }

        match git::has_unpushed_commits(&repo_path) {
            Ok(true) => {
                repos_to_push.push(repo.clone());
            }
            Ok(false) => {
                // Skip repos with nothing to push silently
            }
            Err(e) => {
                ui::print_warning(&format!(
                    "{}: Error checking push status - {}",
                    repo.name, e
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
    for repo in &repos_to_push {
        let repo_name = &repo.name;
        let repo_path = resolve_repo_path(&view_context.view_root, repo);

        ui::print_info(&format!("Pushing commits in {repo_name}"));
        match git::push(&repo_path) {
            Ok(()) => {
                pushed_repos.push(repo_name.clone());
            }
            Err(e) => {
                ui::print_error(&format!("Failed to push {repo_name}: {e}"));

                // For push failures, we can't really rollback, but we can inform the user
                ui::print_error("Push operation stopped due to failure");
                ui::print_info("Common solutions:");
                ui::print_info("   • Pull latest changes: git pull");
                ui::print_info("   • Check remote permissions");
                ui::print_info("   • Verify network connection");
                ui::print_info(&format!(
                    "Successfully pushed repositories: {}",
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
            "Successfully pushed {} repositories: {}",
            pushed_repos.len(),
            pushed_repos.join(", ")
        ));
    }

    Ok(())
}

// Helper functions

#[derive(Debug)]
struct ViewContext {
    viewset_name: String,
    view_root: std::path::PathBuf,
    view_name: String,
    active_repos: Vec<models::Repository>,
}

fn load_view_context(current_dir: &Path) -> Result<ViewContext> {
    // Check if current directory is a view (parent contains .viewyard-repos.json)
    if let Some(parent) = current_dir.parent() {
        let repos_file = parent.join(".viewyard-repos.json");
        if repos_file.exists() {
            let viewset_name = parent
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let view_name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Load and validate repository list from viewset
            let active_repos = load_and_validate_repos(&repos_file).unwrap_or_else(|e| {
                ui::print_error(&format!("Configuration validation failed: {e}"));
                ui::print_info("To fix this:");
                ui::print_info("   • Check the JSON syntax in .viewyard-repos.json");
                ui::print_info("   • Ensure all repositories have 'name' and 'url' fields");
                ui::print_info("   • Use 'cat .viewyard-repos.json' to inspect the file");
                Vec::new()
            });

            return Ok(ViewContext {
                viewset_name,
                view_root: current_dir.to_path_buf(),
                view_name,
                active_repos,
            });
        }
    }

    // Provide detailed context about where the user is and what's expected
    let current_path = current_dir.display();
    let parent_info = current_dir.parent().map_or_else(
        || "No parent directory found".to_string(),
        |parent| format!("Parent directory: {}", parent.display()),
    );

    ui::show_error_with_help(
        "Workspace commands must be run from within a view directory",
        &[
            &format!("Current directory: {current_path}"),
            &parent_info,
            "Expected structure: <viewset>/<view>/",
            "Example: cd my-project/feature-123",
            "Create a view: viewyard view create feature-123",
            "List viewsets: find . -maxdepth 2 -name '.viewyard-repos.json' -exec dirname {} \\;",
        ],
    );
    anyhow::bail!("Not in a view directory")
}

fn get_repo_status(repo_path: &Path, repo_name: &str) -> Result<Option<String>> {
    // Get current branch
    let branch = git::get_current_branch(repo_path)
        .with_context(|| format!("Failed to get current branch for repository '{repo_name}'"))?;

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
                    status_parts.push(format!("{change_count} changes"));
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
                        status_parts.push(format!("{count} commits ahead"));
                    }
                }
            }
            Err(_) => {
                status_parts.push("commits ahead".to_string());
            }
        }
    }

    if stash_count > 0 {
        status_parts.push(format!("{stash_count} stashes"));
    }

    let status_summary = if status_parts.is_empty() {
        "clean".to_string()
    } else {
        status_parts.join(", ")
    };

    let icon = if has_changes { "!" } else { "→" };

    Ok(Some(format!(
        "{icon} {repo_name} ({branch}) - {status_summary}"
    )))
}

fn check_branch_consistency(repo_branches: &[(String, String)]) {
    if repo_branches.len() <= 1 {
        return;
    }

    // Group repos by branch
    let mut branch_groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (repo, branch) in repo_branches {
        branch_groups
            .entry(branch.clone())
            .or_default()
            .push(repo.clone());
    }

    if branch_groups.len() > 1 {
        ui::print_warning("Branch mismatch detected:");
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

fn show_status_summary(
    clean_count: usize,
    dirty_count: usize,
    ahead_count: usize,
    repo_branches: &[(String, String)],
) {
    let total = clean_count + dirty_count;
    let mut summary_parts = Vec::new();

    if clean_count > 0 {
        summary_parts.push(format!("{clean_count} clean"));
    }
    if dirty_count > 0 {
        summary_parts.push(format!("{dirty_count} dirty"));
    }
    if ahead_count > 0 {
        summary_parts.push(format!("{ahead_count} ahead"));
    }

    let status_summary = if summary_parts.is_empty() {
        "all clean".to_string()
    } else {
        summary_parts.join(", ")
    };

    // Check if all repos are on the same branch
    let branch_consistency = if repo_branches.len() <= 1 {
        String::new()
    } else {
        let first_branch = &repo_branches[0].1;
        if repo_branches
            .iter()
            .all(|(_, branch)| branch == first_branch)
        {
            format!(" | All on {first_branch} ✓")
        } else {
            " | Mixed branches !".to_string()
        }
    };

    ui::print_info(&format!(
        "{total} repos: {status_summary}{branch_consistency}"
    ));
}

fn validate_branch_synchronization(view_context: &ViewContext) -> Result<()> {
    let mut branches = std::collections::HashMap::new();
    let mut errors = Vec::new();

    // Check branch for each repository
    for repo in &view_context.active_repos {
        let repo_path = resolve_repo_path(&view_context.view_root, repo);
        let dir_name = repo_directory_name(repo);

        if !repo_path.exists() {
            errors.push(format!(
                "Repository '{}' directory '{}' not found",
                repo.name, dir_name
            ));
            continue;
        }

        if !git::is_git_repo(&repo_path) {
            errors.push(format!(
                "'{}' (directory '{}') is not a git repository",
                repo.name, dir_name
            ));
            continue;
        }

        match git::get_current_branch(&repo_path) {
            Ok(branch) => {
                branches.insert(repo.name.clone(), branch);
            }
            Err(e) => {
                errors.push(format!("Failed to get branch for '{}': {}", repo.name, e));
            }
        }
    }

    // Report any errors
    if !errors.is_empty() {
        for error in &errors {
            ui::print_warning(&format!("{error}"));
        }
        anyhow::bail!("Cannot validate branch synchronization due to repository errors");
    }

    // Check if all repositories are on the same branch
    let expected_branch = &view_context.view_name;
    let mut mismatched_repos = Vec::new();

    for (repo_name, actual_branch) in &branches {
        if actual_branch != expected_branch {
            mismatched_repos.push((repo_name.clone(), actual_branch.clone()));
        }
    }

    if !mismatched_repos.is_empty() {
        ui::show_error_with_help(
            "Branch synchronization error: Not all repositories are on the expected branch",
            &[
                &format!("Expected branch: '{expected_branch}'"),
                "Mismatched repositories:",
            ],
        );

        for (repo_name, actual_branch) in &mismatched_repos {
            println!("  - {repo_name}: on '{actual_branch}' (should be '{expected_branch}')");
        }

        ui::show_error_with_help(
            "",
            &[
                "To fix this, checkout the correct branch in each repository:",
                &format!("  cd <repo> && git checkout {expected_branch}"),
                "Or create the branch if it doesn't exist:",
                &format!("  cd <repo> && git checkout -b {expected_branch}"),
            ],
        );

        anyhow::bail!("Branch synchronization failed");
    }

    ui::print_info(&format!(
        "✓ All repositories are synchronized on branch '{expected_branch}'"
    ));
    Ok(())
}
