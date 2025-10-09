use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod commands;
mod error_handling;
mod git;
mod github;
mod interactive;
mod models;
mod search;
mod ui;

use commands::workspace;
use github::GitHubService;
use interactive::InteractiveSelector;

/// Validate and load repository configuration from JSON file
fn load_and_validate_repos(repos_file: &std::path::Path) -> Result<Vec<models::Repository>> {
    let repos_json = std::fs::read_to_string(repos_file).with_context(|| {
        format!(
            "Failed to read configuration file: {}",
            repos_file.display()
        )
    })?;

    let mut repositories: Vec<models::Repository> = serde_json::from_str(&repos_json)
        .with_context(|| {
            format!(
                "Invalid JSON in configuration file: {}\n\
                Expected format: array of repository objects with 'name', 'url', 'is_private', and 'source' fields",
                repos_file.display()
            )
        })?;

    // Transform URLs to use SSH host aliases if available
    for repo in &mut repositories {
        if let Some(ref account) = repo.account {
            repo.url = git::transform_github_url_for_account(&repo.url, account);
        } else if let Ok(account) = git::extract_account_from_source(&repo.source) {
            repo.url = git::transform_github_url_for_account(&repo.url, &account);
        }
    }

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

#[derive(Parser)]
#[command(name = "viewyard")]
#[command(about = "Multi-repository workspace management tool")]
#[command(version)]
#[command(
    long_about = "The refreshingly unoptimized alternative to monorepos.\n\nA clean, simple workspace for coordinated development across multiple repositories using task-based views with dynamic repository discovery."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Viewset management commands
    Viewset {
        #[command(subcommand)]
        action: ViewsetCommand,
    },

    /// View management commands
    View {
        #[command(subcommand)]
        action: ViewCommand,
    },

    // Workspace commands (work from within a view directory)
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

#[derive(Subcommand)]
enum ViewsetCommand {
    /// Create a new viewset with repository selection
    Create {
        /// Name of the viewset directory to create
        name: String,
        /// GitHub account to search repositories from
        #[arg(long)]
        account: Option<String>,
    },
    /// Update an existing viewset by adding new repositories
    Update {
        /// GitHub account to search repositories from
        #[arg(short, long)]
        account: Option<String>,
    },
}

#[derive(Subcommand)]
enum ViewCommand {
    /// Create a new view/branch within the current viewset
    Create {
        /// Name of the view/branch to create
        name: String,
    },
    /// Update an existing view to include new repositories from the viewset
    Update {
        /// Name of the view to update (defaults to current view)
        name: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Viewset { action } => handle_viewset_command(action),
        Commands::View { action } => handle_view_command(action),

        // Workspace commands
        Commands::Status => workspace::handle_command(workspace::WorkspaceCommand::Status),
        Commands::Rebase => workspace::handle_command(workspace::WorkspaceCommand::Rebase),
        Commands::CommitAll { message } => {
            workspace::handle_command(workspace::WorkspaceCommand::CommitAll { message })
        }
        Commands::PushAll => workspace::handle_command(workspace::WorkspaceCommand::PushAll),
    }
}

fn handle_viewset_command(action: ViewsetCommand) -> Result<()> {
    match action {
        ViewsetCommand::Create { name, account } => create_viewset(&name, account.as_deref()),
        ViewsetCommand::Update { account } => update_viewset(account.as_deref()),
    }
}

fn handle_view_command(action: ViewCommand) -> Result<()> {
    match action {
        ViewCommand::Create { name } => create_view(&name),
        ViewCommand::Update { name } => update_view(name.as_deref()),
    }
}

fn create_viewset(name: &str, account: Option<&str>) -> Result<()> {
    ui::print_info(&format!("Creating viewset: {name}"));

    // Check if git is available
    git::check_git_availability()?;

    let viewset_path = std::env::current_dir()?.join(name);

    if viewset_path.exists() {
        ui::show_error_with_help(
            &format!("Directory '{name}' already exists"),
            &["Choose a different name or remove the existing directory"],
        );
        return Err(anyhow::anyhow!("Directory already exists"));
    }

    // Discover repositories
    let Ok(repositories) = discover_repositories_for_viewset(account) else {
        return create_empty_viewset(&viewset_path, name, "when GitHub CLI is set up");
    };

    // Interactive repository selection
    let selector = InteractiveSelector::new();
    let selected_repos = selector.select_repositories(&repositories)?;

    if selected_repos.is_empty() {
        ui::print_info("No repositories selected. Creating empty viewset.");
        return create_empty_viewset(&viewset_path, name, "later");
    }

    // Confirm selection
    if !InteractiveSelector::confirm_selection(&selected_repos)? {
        ui::print_info("Repository selection cancelled.");
        return Ok(());
    }

    // Create viewset directory
    std::fs::create_dir_all(&viewset_path)?;
    ui::print_success(&format!(
        "✓ Created viewset directory: {}",
        viewset_path.display()
    ));

    // Store repository list for the viewset
    let repos_file = viewset_path.join(".viewyard-repos.json");
    let repos_json = serde_json::to_string_pretty(&selected_repos)?;
    std::fs::write(&repos_file, repos_json)?;

    ui::print_success(&format!(
        "Viewset '{}' created successfully with {} repositories!",
        name,
        selected_repos.len()
    ));
    ui::print_info(&format!("Navigate to: cd {name}"));
    ui::print_info("Run 'viewyard view create <view-name>' to create your first view");

    Ok(())
}

/// Create an empty viewset directory with helpful instructions
fn create_empty_viewset(viewset_path: &std::path::Path, name: &str, when: &str) -> Result<()> {
    std::fs::create_dir_all(viewset_path)?;
    ui::print_success(&format!(
        "✓ Created empty viewset directory: {}",
        viewset_path.display()
    ));
    ui::print_info(&format!("Navigate to: cd {name}"));
    ui::print_info(&format!("Manually edit .viewyard-repos.json to add repositories {when}"));
    ui::print_info("Then run 'viewyard view create <view-name>' to create your first view");
    Ok(())
}

/// Load repositories from a viewset with validation
fn load_viewset_repositories(viewset_root: &std::path::Path) -> Result<Vec<models::Repository>> {
    let repos_file = viewset_root.join(".viewyard-repos.json");
    if !repos_file.exists() {
        ui::show_error_with_help(
            "No repositories found in this viewset",
            &[
                "Manually edit .viewyard-repos.json to add repositories to this viewset",
                "Or create a new viewset with 'viewyard viewset create <name>'",
            ],
        );
        anyhow::bail!("No repositories in viewset");
    }

    let repositories = load_and_validate_repos(&repos_file)?;

    if repositories.is_empty() {
        ui::show_error_with_help(
            "No repositories found in this viewset",
            &["Manually edit .viewyard-repos.json to add repositories to this viewset"],
        );
        anyhow::bail!("No repositories in viewset");
    }

    Ok(repositories)
}

fn create_view(view_name: &str) -> Result<()> {
    ui::print_info(&format!("Creating view: {view_name}"));

    // Check if git is available
    git::check_git_availability()?;

    // Detect viewset context
    let viewset_context = detect_viewset_context()?;
    let view_path = viewset_context.viewset_root.join(view_name);

    if view_path.exists() {
        ui::show_error_with_help(
            &format!("View '{view_name}' already exists"),
            &["Choose a different view name or remove the existing view directory"],
        );
        return Err(anyhow::anyhow!("View already exists"));
    }

    // Load repository list from viewset
    let repositories = load_viewset_repositories(&viewset_context.viewset_root)?;

    // Create temporary directory for atomic operation
    let temp_view_path = view_path.with_extension("tmp");

    // Ensure temp directory doesn't exist from previous failed operation
    if temp_view_path.exists() {
        std::fs::remove_dir_all(&temp_view_path)?;
    }

    std::fs::create_dir_all(&temp_view_path)?;
    ui::print_info(&format!(
        "✓ Created temporary view directory: {}",
        temp_view_path.display()
    ));

    // Clone repositories and create/checkout branches to temporary directory
    ui::print_info("Cloning repositories and setting up branches...");

    // Track success for cleanup on failure

    for repo in &repositories {
        ui::print_info(&format!(
            "  Setting up {} on branch '{}'",
            repo.name, view_name
        ));

        match clone_and_setup_branch(repo, &temp_view_path, view_name) {
            Ok(()) => {
                // Repository cloned successfully
            }
            Err(e) => {
                // Cleanup temporary directory on any failure
                ui::print_error(&format!("Failed to setup {}: {}", repo.name, e));
                ui::print_info("Cleaning up temporary files...");
                if let Err(cleanup_err) = std::fs::remove_dir_all(&temp_view_path) {
                    ui::print_warning(&format!(
                        "Failed to cleanup temporary directory: {cleanup_err}"
                    ));
                }
                return Err(e.context(format!("Failed to setup repository '{}'", repo.name)));
            }
        }
    }

    // All operations succeeded - atomically move temp directory to final location
    std::fs::rename(&temp_view_path, &view_path).context("Failed to finalize view creation")?;

    ui::print_success(&format!(
        "View '{}' created successfully with {} repositories!",
        view_name,
        repositories.len()
    ));
    ui::print_info(&format!("Navigate to: cd {view_name}"));
    ui::print_info("All repositories are on the same branch for coordinated development");
    ui::print_info("Run 'viewyard status' to see repository status");

    Ok(())
}

// Context detection structures
#[derive(Debug)]
struct ViewsetContext {
    viewset_root: std::path::PathBuf,
}

fn detect_viewset_context() -> Result<ViewsetContext> {
    let current_dir = std::env::current_dir()?;

    // Check if current directory is a viewset root (contains .viewyard-repos.json)
    let repos_file = current_dir.join(".viewyard-repos.json");
    if repos_file.exists() {
        return Ok(ViewsetContext {
            viewset_root: current_dir,
        });
    }

    // Check if current directory is a view (parent contains .viewyard-repos.json)
    if let Some(parent) = current_dir.parent() {
        let repos_file = parent.join(".viewyard-repos.json");
        if repos_file.exists() {
            return Ok(ViewsetContext {
                viewset_root: parent.to_path_buf(),
            });
        }
    }

    // Provide detailed context about where the user is and what's expected
    let current_path = current_dir.display();

    ui::show_error_with_help(
        "Viewset commands must be run from within a viewset directory",
        &[
            &format!("Current directory: {current_path}"),
            "Expected: directory containing .viewyard-repos.json",
            "Create a viewset: viewyard viewset create my-project",
            "Then navigate: cd my-project",
            "List existing viewsets: find . -maxdepth 2 -name '.viewyard-repos.json' -exec dirname {} \\;",
        ],
    );
    Err(anyhow::anyhow!("Not in a viewset directory"))
}

fn clone_and_setup_branch(
    repo: &models::Repository,
    view_path: &std::path::Path,
    branch_name: &str,
) -> Result<()> {
    let repo_path = view_path.join(&repo.name);

    // Clone repository (full clone for complete git functionality)
    let output = std::process::Command::new("git")
        .args(["clone", &repo.url, &repo.name])
        .current_dir(view_path)
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error_handling::handle_clone_error(&repo.name, &stderr)?;
    }

    ui::print_info(&format!("  Cloned {}", repo.name));

    // Configure git user identity for this repository
    git::validate_repository_for_operations(&repo_path, repo)
        .with_context(|| format!("Failed to configure git user for repository: {}", repo.name))?;

    // Create and checkout branch
    setup_branch_in_repo(&repo_path, branch_name)?;

    Ok(())
}

fn setup_branch_in_repo(repo_path: &std::path::Path, branch_name: &str) -> Result<()> {
    // Check if branch already exists
    let check_output = std::process::Command::new("git")
        .args(["branch", "--list", branch_name])
        .current_dir(repo_path)
        .output()
        .context("Failed to check if branch exists")?;

    let branch_exists = !String::from_utf8_lossy(&check_output.stdout)
        .trim()
        .is_empty();

    if branch_exists {
        // Checkout existing branch
        let output = std::process::Command::new("git")
            .args(["checkout", branch_name])
            .current_dir(repo_path)
            .output()
            .context("Failed to checkout existing branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error_handling::handle_checkout_error(branch_name, repo_path, &stderr)?;
        }
        ui::print_info(&format!("    Checked out existing branch '{branch_name}'"));
    } else {
        // Create new branch from current default branch
        let output = std::process::Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(repo_path)
            .output()
            .context("Failed to create new branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error_handling::handle_branch_creation_error(branch_name, repo_path, &stderr)?;
        }
        ui::print_info(&format!(
            "    Created and checked out new branch '{branch_name}'"
        ));
    }

    Ok(())
}

fn update_view(view_name: Option<&str>) -> Result<()> {
    // Check if git is available
    git::check_git_availability()?;

    // Detect viewset context
    let viewset_context = detect_viewset_context()?;

    // Determine view name - use provided name or detect from current directory
    let target_view_name = if let Some(name) = view_name {
        name.to_string()
    } else {
        // Try to detect current view from directory name
        std::env::current_dir()?
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Could not determine view name from current directory"))?
            .to_string()
    };

    let view_path = viewset_context.viewset_root.join(&target_view_name);

    // Check if view exists
    if !view_path.exists() {
        ui::show_error_with_help(
            &format!("View '{target_view_name}' does not exist"),
            &[
                "Create the view first: viewyard view create <view-name>",
                "Or specify an existing view: viewyard view update <existing-view-name>",
            ],
        );
        return Err(anyhow::anyhow!("View does not exist"));
    }

    ui::print_info(&format!("Updating view: {target_view_name}"));

    // Load repository list from viewset
    let Ok(repositories) = load_viewset_repositories(&viewset_context.viewset_root) else {
        ui::print_info("No repositories in viewset - nothing to update");
        return Ok(());
    };

    // Determine which repositories are missing from the current view
    let missing_repos = find_missing_repositories(&repositories, &view_path);

    if missing_repos.is_empty() {
        ui::print_success("View is already up to date - all repositories are present");
        return Ok(());
    }

    ui::print_info(&format!(
        "Found {} missing repositories to add: {}",
        missing_repos.len(),
        missing_repos
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    // Clone and setup missing repositories directly in the view
    ui::print_info("Adding missing repositories...");

    for repo in &missing_repos {
        ui::print_info(&format!(
            "  Setting up {} on branch '{}'",
            repo.name, target_view_name
        ));

        match clone_and_setup_repository_in_view(repo, &view_path, &target_view_name) {
            Ok(()) => {
                ui::print_info(&format!("  ✓ Added {}", repo.name));
            }
            Err(e) => {
                ui::print_error(&format!("Failed to add {}: {}", repo.name, e));
                return Err(e.context(format!("Failed to add repository '{}'", repo.name)));
            }
        }
    }

    ui::print_success(&format!(
        "View '{}' updated successfully! Added {} repositories.",
        target_view_name,
        missing_repos.len()
    ));

    Ok(())
}

/// Find repositories that are missing from the current view
fn find_missing_repositories(
    all_repos: &[models::Repository],
    view_path: &std::path::Path,
) -> Vec<models::Repository> {
    let mut missing_repos = Vec::new();

    for repo in all_repos {
        let repo_path = view_path.join(&repo.name);
        if !repo_path.exists() {
            missing_repos.push(repo.clone());
        }
    }

    missing_repos
}

/// Clone and setup a single repository directly in an existing view
fn clone_and_setup_repository_in_view(
    repo: &models::Repository,
    view_path: &std::path::Path,
    branch_name: &str,
) -> Result<()> {
    let repo_path = view_path.join(&repo.name);

    // Ensure the repository directory doesn't already exist
    if repo_path.exists() {
        return Ok(()); // Already exists, nothing to do
    }

    // Clone repository directly into the view
    let output = std::process::Command::new("git")
        .args(["clone", &repo.url, &repo.name])
        .current_dir(view_path)
        .output()
        .context("Failed to execute git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Provide specific recovery guidance based on error type
        if stderr.contains("Permission denied") || stderr.contains("publickey") {
            ui::print_error(&format!("SSH authentication failed for {}", repo.name));
            ui::print_info("SSH key issues detected:");
            ui::print_info("   • Test SSH connection: ssh -T git@github.com");
            ui::print_info(
                "   • Add SSH key to GitHub: gh auth refresh -h github.com -s admin:public_key",
            );
            anyhow::bail!("SSH authentication failed for repository '{}'", repo.name);
        } else if stderr.contains("not found") || stderr.contains("does not exist") {
            ui::print_error(&format!("Repository not found: {}", repo.name));
            ui::print_info("Repository access issues:");
            ui::print_info(&format!(
                "   • Verify repository exists: gh repo view {}",
                repo.name
            ));
            ui::print_info("   • Check repository URL in .viewyard-repos.json");
            ui::print_info("   • Ensure you have access to this repository");
            anyhow::bail!("Repository '{}' not found or inaccessible", repo.name);
        }

        anyhow::bail!("Failed to clone repository '{}': {}", repo.name, stderr);
    }

    // Configure git user for the repository
    if let Some(ref account) = repo.account {
        git::validate_and_configure_git_user(&repo_path, account)?;
    } else if let Ok(account) = git::extract_account_from_source(&repo.source) {
        git::validate_and_configure_git_user(&repo_path, &account)?;
    }

    // Setup branch in the newly cloned repository
    setup_branch_in_repo(&repo_path, branch_name)?;

    Ok(())
}

/// Discover repositories from GitHub based on account preference
fn discover_repositories_for_viewset(account: Option<&str>) -> Result<Vec<models::Repository>> {
    // Check GitHub CLI availability
    if !GitHubService::check_availability()? {
        ui::show_error_with_help(
            "GitHub CLI is not available or not authenticated",
            &[
                "Install GitHub CLI: https://cli.github.com/",
                "Then authenticate: gh auth login",
                "Or manually edit .viewyard-repos.json to add repositories",
            ],
        );
        anyhow::bail!("GitHub CLI not available");
    }

    // Discover repositories
    ui::print_info("Discovering repositories from GitHub...");

    let repositories = if let Some(specific_account) = account {
        GitHubService::discover_repositories_from_account(specific_account)?
    } else {
        GitHubService::discover_all_repositories()?
    };

    if repositories.is_empty() {
        ui::print_warning("No repositories found");
        anyhow::bail!("No repositories found");
    }

    Ok(repositories)
}

/// Filter out repositories that already exist in the viewset
fn filter_existing_repositories(
    all_repos: &[models::Repository],
    existing_repos: &[models::Repository],
) -> Vec<models::Repository> {
    let existing_names: std::collections::HashSet<&str> =
        existing_repos.iter().map(|r| r.name.as_str()).collect();

    all_repos
        .iter()
        .filter(|repo| !existing_names.contains(repo.name.as_str()))
        .cloned()
        .collect()
}

/// Interactive repository selection with context about existing repositories
fn select_repositories_for_update(
    available_repos: &[models::Repository],
    existing_repos: &[models::Repository],
) -> Result<Vec<models::Repository>> {
    if available_repos.is_empty() {
        ui::print_info("All available repositories are already in the viewset.");
        return Ok(Vec::new());
    }

    ui::print_info(&format!(
        "Current viewset has {} repositories. Found {} new repositories available to add.",
        existing_repos.len(),
        available_repos.len()
    ));

    if !existing_repos.is_empty() {
        println!("Existing repositories in viewset:");
        for repo in existing_repos {
            println!("  • {}", repo.name);
        }
        println!();
    }

    // Interactive repository selection
    let selector = InteractiveSelector::new();
    let selected_repos =
        selector.select_repositories_with_existing(available_repos, existing_repos)?;

    if selected_repos.is_empty() {
        ui::print_info("No new repositories selected.");
        return Ok(Vec::new());
    }

    // Confirm selection
    if !InteractiveSelector::confirm_selection(&selected_repos)? {
        ui::print_info("Repository selection cancelled.");
        return Ok(Vec::new());
    }

    Ok(selected_repos)
}

fn update_viewset(account: Option<&str>) -> Result<()> {
    ui::print_info("Updating viewset with new repositories");

    // Check if git is available
    git::check_git_availability()?;

    // Detect viewset context (must be in viewset root for update)
    let current_dir = std::env::current_dir()?;
    let repos_file = current_dir.join(".viewyard-repos.json");

    if !repos_file.exists() {
        ui::show_error_with_help(
            "Not in a viewset directory",
            &[
                &format!("Current directory: {}", current_dir.display()),
                "Expected: directory containing .viewyard-repos.json",
                "Navigate to a viewset directory first",
                "Or create a new viewset: viewyard viewset create <name>",
            ],
        );
        return Err(anyhow::anyhow!("Not in a viewset directory"));
    }

    // Load existing repositories
    let existing_repos = load_and_validate_repos(&repos_file)?;

    // Discover available repositories
    let Ok(all_repos) = discover_repositories_for_viewset(account) else {
        ui::print_info("Falling back to manual repository management.");
        ui::print_info("Edit .viewyard-repos.json manually to add repositories.");
        return Ok(());
    };

    // Filter out repositories that already exist
    let available_repos = filter_existing_repositories(&all_repos, &existing_repos);

    // Interactive selection of new repositories
    let selected_repos = select_repositories_for_update(&available_repos, &existing_repos)?;

    if selected_repos.is_empty() {
        ui::print_success("No changes made to viewset.");
        return Ok(());
    }

    // Merge existing and new repositories
    let mut updated_repos = existing_repos;
    updated_repos.extend(selected_repos.iter().cloned());

    // Update the repository configuration file
    let repos_json = serde_json::to_string_pretty(&updated_repos)?;
    std::fs::write(&repos_file, repos_json)?;

    ui::print_success(&format!(
        "Viewset updated successfully! Added {} new repositories.",
        selected_repos.len()
    ));

    // Show what was added
    ui::print_info("Added repositories:");
    for repo in &selected_repos {
        ui::print_info(&format!("  • {}", repo.name));
    }

    ui::print_info("");
    ui::print_info("Next steps:");
    ui::print_info("  • Update existing views: viewyard view update");
    ui::print_info("  • Or create a new view: viewyard view create <view-name>");

    Ok(())
}
