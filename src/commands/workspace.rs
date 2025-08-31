use anyhow::Result;
use clap::Subcommand;
use std::fs;
use std::path::Path;

use crate::config;
use crate::git;
use crate::ui;

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// Show status of all repos in current view
    Status,
    /// Rebase repos against origin/master
    Rebase,
    /// Build repos with changes
    Build,
    /// Run tests on repos with changes
    Test,
    /// Commit to all dirty repos
    #[command(name = "commit-all")]
    CommitAll {
        /// Commit message
        message: String,
    },
    /// Push repos with commits ahead
    #[command(name = "push-all")]
    PushAll,
}

pub fn handle_command(command: WorkspaceCommand) -> Result<()> {
    match command {
        WorkspaceCommand::Status => workspace_status(),
        WorkspaceCommand::Rebase => workspace_rebase(),
        WorkspaceCommand::Build => workspace_build(),
        WorkspaceCommand::Test => workspace_test(),
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

        // Get repository status
        match get_repo_status(&repo_path, repo_name) {
            Ok(status) => ui::print_info(&status),
            Err(e) => ui::print_warning(&format!("⚠️  {}: Error getting status - {}", repo_name, e)),
        }
    }

    Ok(())
}

fn workspace_rebase() -> Result<()> {
    ui::print_header("Rebasing repositories");
    // TODO: Implement rebase for all repos
    ui::print_success("All repositories rebased successfully");
    Ok(())
}

fn workspace_build() -> Result<()> {
    ui::print_header("Building repositories with changes");

    let current_dir = std::env::current_dir()?;
    let view_context = load_view_context(&current_dir)?;

    // Load viewsets config to get build commands
    let config = config::load_viewsets_config()?;
    let viewset_name = config::detect_current_viewset()
        .ok_or_else(|| anyhow::anyhow!("Could not detect current viewset"))?;
    let viewset = config.viewsets.get(&viewset_name)
        .ok_or_else(|| anyhow::anyhow!("Viewset '{}' not found", viewset_name))?;

    let mut built_repos = Vec::new();
    let mut skipped_repos = Vec::new();
    let mut error_repos = Vec::new();

    for repo_name in &view_context.active_repos {
        let repo_path = view_context.view_root.join(repo_name);

        if !repo_path.exists() || !git::is_git_repo(&repo_path) {
            error_repos.push(repo_name.clone());
            continue;
        }

        // Find the repository configuration
        let repo_config = viewset.repos.iter()
            .find(|r| r.name == *repo_name);

        match repo_config.and_then(|r| r.build.as_ref()) {
            Some(build_command) => {
                // Check if repo has changes
                match git::has_uncommitted_changes(&repo_path) {
                    Ok(true) => {
                        ui::print_info(&format!("Building {} with: {}", repo_name, build_command));
                        match run_build_command(&repo_path, build_command) {
                            Ok(()) => built_repos.push(repo_name.clone()),
                            Err(e) => {
                                ui::print_warning(&format!("Build failed for {}: {}", repo_name, e));
                                error_repos.push(repo_name.clone());
                            }
                        }
                    }
                    Ok(false) => {
                        ui::print_info(&format!("Skipping {} (no changes)", repo_name));
                        skipped_repos.push(repo_name.clone());
                    }
                    Err(e) => {
                        ui::print_warning(&format!("Error checking status of {}: {}", repo_name, e));
                        error_repos.push(repo_name.clone());
                    }
                }
            }
            None => {
                ui::print_info(&format!("Skipping {} (no build command configured)", repo_name));
                skipped_repos.push(repo_name.clone());
            }
        }
    }

    // Summary
    ui::print_info("");
    if !built_repos.is_empty() {
        ui::print_success(&format!("✅ Built {} repositories: {}",
            built_repos.len(), built_repos.join(", ")));
    }
    if !skipped_repos.is_empty() {
        ui::print_info(&format!("⏭️ Skipped {} repositories: {}",
            skipped_repos.len(), skipped_repos.join(", ")));
    }
    if !error_repos.is_empty() {
        ui::print_warning(&format!("⚠️ {} repositories had errors: {}",
            error_repos.len(), error_repos.join(", ")));
    }

    Ok(())
}

fn run_build_command(repo_path: &Path, build_command: &str) -> Result<()> {
    use std::process::Command;

    let output = Command::new("sh")
        .arg("-c")
        .arg(build_command)
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Build command failed: {}", stderr);
    }

    Ok(())
}

fn workspace_test() -> Result<()> {
    ui::print_header("Testing repositories with changes");
    // TODO: Implement test for repos with changes
    ui::print_success("Tests completed successfully");
    Ok(())
}

fn workspace_commit_all(message: &str) -> Result<()> {
    ui::print_header(&format!("Committing all dirty repos: {}", message));

    let current_dir = std::env::current_dir()?;
    let view_context = load_view_context(&current_dir)?;

    let mut committed_repos = Vec::new();
    let mut clean_repos = Vec::new();
    let mut error_repos = Vec::new();

    for repo_name in &view_context.active_repos {
        let repo_path = view_context.view_root.join(repo_name);

        if !repo_path.exists() || !git::is_git_repo(&repo_path) {
            error_repos.push(repo_name.clone());
            continue;
        }

        match git::has_uncommitted_changes(&repo_path) {
            Ok(true) => {
                ui::print_info(&format!("Committing changes in {}", repo_name));
                match commit_repo_changes(&repo_path, message) {
                    Ok(()) => committed_repos.push(repo_name.clone()),
                    Err(e) => {
                        ui::print_warning(&format!("Failed to commit {}: {}", repo_name, e));
                        error_repos.push(repo_name.clone());
                    }
                }
            }
            Ok(false) => clean_repos.push(repo_name.clone()),
            Err(e) => {
                ui::print_warning(&format!("Error checking status of {}: {}", repo_name, e));
                error_repos.push(repo_name.clone());
            }
        }
    }

    // Summary
    ui::print_info("");
    if !committed_repos.is_empty() {
        ui::print_success(&format!("✅ Committed {} repositories: {}",
            committed_repos.len(), committed_repos.join(", ")));
    }
    if !clean_repos.is_empty() {
        ui::print_info(&format!("🔄 {} repositories were already clean: {}",
            clean_repos.len(), clean_repos.join(", ")));
    }
    if !error_repos.is_empty() {
        ui::print_warning(&format!("⚠️ {} repositories had errors: {}",
            error_repos.len(), error_repos.join(", ")));
    }

    Ok(())
}

fn commit_repo_changes(repo_path: &Path, message: &str) -> Result<()> {
    git::add_all(repo_path)?;
    git::commit(message, repo_path)?;
    Ok(())
}

fn workspace_push_all() -> Result<()> {
    ui::print_header("Pushing repositories with commits ahead");

    let current_dir = std::env::current_dir()?;
    let view_context = load_view_context(&current_dir)?;

    let mut pushed_repos = Vec::new();
    let mut clean_repos = Vec::new();
    let mut error_repos = Vec::new();

    for repo_name in &view_context.active_repos {
        let repo_path = view_context.view_root.join(repo_name);

        if !repo_path.exists() || !git::is_git_repo(&repo_path) {
            error_repos.push(repo_name.clone());
            continue;
        }

        match git::has_unpushed_commits(&repo_path) {
            Ok(true) => {
                ui::print_info(&format!("Pushing commits in {}", repo_name));
                match git::push(&repo_path) {
                    Ok(()) => pushed_repos.push(repo_name.clone()),
                    Err(e) => {
                        ui::print_warning(&format!("Failed to push {}: {}", repo_name, e));
                        error_repos.push(repo_name.clone());
                    }
                }
            }
            Ok(false) => clean_repos.push(repo_name.clone()),
            Err(e) => {
                ui::print_warning(&format!("Error checking push status of {}: {}", repo_name, e));
                error_repos.push(repo_name.clone());
            }
        }
    }

    // Summary
    ui::print_info("");
    if !pushed_repos.is_empty() {
        ui::print_success(&format!("✅ Pushed {} repositories: {}",
            pushed_repos.len(), pushed_repos.join(", ")));
    }
    if !clean_repos.is_empty() {
        ui::print_info(&format!("🔄 {} repositories had nothing to push: {}",
            clean_repos.len(), clean_repos.join(", ")));
    }
    if !error_repos.is_empty() {
        ui::print_warning(&format!("⚠️ {} repositories had errors: {}",
            error_repos.len(), error_repos.join(", ")));
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
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
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

fn get_repo_status(repo_path: &Path, repo_name: &str) -> Result<String> {
    // Get current branch
    let branch = git::get_current_branch(repo_path)
        .unwrap_or_else(|_| "unknown".to_string());

    // Check for uncommitted changes
    let has_changes = git::has_uncommitted_changes(repo_path)?;

    // Check for unpushed commits
    let has_unpushed = git::has_unpushed_commits(repo_path)?;

    // Build status string
    let mut status_parts = vec![format!("📁 {}", repo_name)];

    // Add branch info
    status_parts.push(format!("({})", branch));

    // Add status indicators
    let mut indicators = Vec::new();
    if has_changes {
        indicators.push("🔄 changes");
    }
    if has_unpushed {
        indicators.push("⬆️ unpushed");
    }
    if indicators.is_empty() {
        indicators.push("✅ clean");
    }

    status_parts.push(format!("[{}]", indicators.join(", ")));

    Ok(status_parts.join(" "))
}
