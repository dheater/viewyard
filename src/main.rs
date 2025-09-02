use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod commands;
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
}

#[derive(Subcommand)]
enum ViewCommand {
    /// Create a new view/branch within the current viewset
    Create {
        /// Name of the view/branch to create
        name: String,
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
    }
}

fn handle_view_command(action: ViewCommand) -> Result<()> {
    match action {
        ViewCommand::Create { name } => create_view(&name),
    }
}

fn create_viewset(name: &str, account: Option<&str>) -> Result<()> {
    ui::print_info(&format!("📦 Creating viewset: {name}"));

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

    // Check GitHub CLI availability
    if !GitHubService::check_availability()? {
        ui::show_error_with_help(
            "GitHub CLI is not available or not authenticated",
            &[
                "Install GitHub CLI: https://cli.github.com/",
                "Then authenticate: gh auth login",
                "Or create an empty viewset and manually edit .viewyard-repos.json",
            ],
        );

        // Create empty directory as fallback
        std::fs::create_dir_all(&viewset_path)?;
        ui::print_success(&format!(
            "✓ Created empty viewset directory: {}",
            viewset_path.display()
        ));
        ui::print_info(&format!("Navigate to: cd {name}"));
        ui::print_info(
            "Manually edit .viewyard-repos.json to add repositories when GitHub CLI is set up",
        );
        ui::print_info("Then run 'viewyard view create <view-name>' to create your first view");
        return Ok(());
    }

    // Discover repositories
    ui::print_info("🔍 Discovering repositories from GitHub...");

    let repositories = if let Some(specific_account) = account {
        GitHubService::discover_repositories_from_account(specific_account)?
    } else {
        GitHubService::discover_all_repositories()?
    };

    if repositories.is_empty() {
        ui::print_warning("No repositories found");
        std::fs::create_dir_all(&viewset_path)?;
        ui::print_success(&format!(
            "✓ Created empty viewset directory: {}",
            viewset_path.display()
        ));
        ui::print_info(&format!("Navigate to: cd {name}"));
        ui::print_info("Run 'viewyard view create <view-name>' to create your first view");
        return Ok(());
    }

    // Interactive repository selection
    let selector = InteractiveSelector::new();
    let selected_repos = selector.select_repositories(&repositories)?;

    if selected_repos.is_empty() {
        ui::print_info("No repositories selected. Creating empty viewset.");
        std::fs::create_dir_all(&viewset_path)?;
        ui::print_success(&format!(
            "✓ Created empty viewset directory: {}",
            viewset_path.display()
        ));
        ui::print_info(&format!("Navigate to: cd {name}"));
        ui::print_info("Manually edit .viewyard-repos.json to add repositories later");
        ui::print_info("Then run 'viewyard view create <view-name>' to create your first view");
        return Ok(());
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
        "🎉 Viewset '{}' created successfully with {} repositories!",
        name,
        selected_repos.len()
    ));
    ui::print_info(&format!("Navigate to: cd {name}"));
    ui::print_info("Run 'viewyard view create <view-name>' to create your first view");

    Ok(())
}

fn create_view(view_name: &str) -> Result<()> {
    ui::print_info(&format!("🌿 Creating view: {view_name}"));

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
    let repos_file = viewset_context.viewset_root.join(".viewyard-repos.json");
    if !repos_file.exists() {
        ui::show_error_with_help(
            "No repositories found in this viewset",
            &[
                "Manually edit .viewyard-repos.json to add repositories to this viewset",
                "Or create a new viewset with 'viewyard viewset create <name>'",
            ],
        );
        return Err(anyhow::anyhow!("No repositories in viewset"));
    }

    let repositories = load_and_validate_repos(&repos_file)?;

    if repositories.is_empty() {
        ui::show_error_with_help(
            "No repositories found in this viewset",
            &["Manually edit .viewyard-repos.json to add repositories to this viewset"],
        );
        return Err(anyhow::anyhow!("No repositories in viewset"));
    }

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
    ui::print_info("📥 Cloning repositories and setting up branches...");

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
                ui::print_error(&format!("❌ Failed to setup {}: {}", repo.name, e));
                ui::print_info("🧹 Cleaning up temporary files...");
                if let Err(cleanup_err) = std::fs::remove_dir_all(&temp_view_path) {
                    ui::print_warning(&format!(
                        "⚠️  Failed to cleanup temporary directory: {cleanup_err}"
                    ));
                }
                return Err(e.context(format!("Failed to setup repository '{}'", repo.name)));
            }
        }
    }

    // All operations succeeded - atomically move temp directory to final location
    std::fs::rename(&temp_view_path, &view_path).context("Failed to finalize view creation")?;

    ui::print_success(&format!(
        "🎉 View '{}' created successfully with {} repositories!",
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

        // Provide specific recovery guidance based on error type
        if stderr.contains("Permission denied") || stderr.contains("publickey") {
            ui::print_error(&format!("❌ SSH authentication failed for {}", repo.name));
            ui::print_info("🔑 SSH key issues detected:");
            ui::print_info("   • Test SSH connection: ssh -T git@github.com");
            ui::print_info(
                "   • Add SSH key to GitHub: gh auth refresh -h github.com -s admin:public_key",
            );
            ui::print_info("   • Or use HTTPS: git config --global url.\"https://github.com/\".insteadOf git@github.com:");
            anyhow::bail!("SSH authentication failed for repository '{}'", repo.name);
        } else if stderr.contains("not found") || stderr.contains("does not exist") {
            ui::print_error(&format!("❌ Repository not found: {}", repo.name));
            ui::print_info("🔍 Repository access issues:");
            ui::print_info(&format!(
                "   • Verify repository exists: gh repo view {}",
                repo.name
            ));
            ui::print_info("   • Check repository URL in .viewyard-repos.json");
            ui::print_info("   • Ensure you have access to this repository");
            anyhow::bail!("Repository '{}' not found or inaccessible", repo.name);
        } else if stderr.contains("timeout") || stderr.contains("network") {
            ui::print_error(&format!("❌ Network timeout cloning {}", repo.name));
            ui::print_info("🌐 Network issues detected:");
            ui::print_info("   • Check internet connection");
            ui::print_info("   • Try again in a few moments");
            ui::print_info("   • Consider using a VPN if behind corporate firewall");
            anyhow::bail!("Network timeout cloning repository '{}'", repo.name);
        } else if stderr.contains("already exists") {
            ui::print_error(&format!("❌ Directory already exists: {}", repo.name));
            ui::print_info("📁 Directory conflict:");
            ui::print_info(&format!(
                "   • Remove existing directory: rm -rf {}",
                repo.name
            ));
            ui::print_info("   • Or choose a different view name");
            anyhow::bail!("Directory '{}' already exists", repo.name);
        }
        // Generic error with full stderr
        ui::print_error(&format!("❌ Failed to clone {}", repo.name));
        ui::print_info("🔧 Git clone failed:");
        ui::print_info(&format!("   • Error: {}", stderr.trim()));
        ui::print_info("   • Check repository URL and permissions");
        ui::print_info("   • Verify git and network connectivity");
        anyhow::bail!("Failed to clone repository '{}': {}", repo.name, stderr);
    }

    ui::print_info(&format!("  ✓ Cloned {}", repo.name));

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

            if stderr.contains("uncommitted changes") || stderr.contains("would be overwritten") {
                ui::print_error(&format!(
                    "❌ Cannot checkout branch '{branch_name}' - uncommitted changes"
                ));
                ui::print_info("💾 Uncommitted changes detected:");
                ui::print_info(&format!("   • Navigate to: cd {}", repo_path.display()));
                ui::print_info("   • Commit changes: git add . && git commit -m \"Save work\"");
                ui::print_info("   • Or stash changes: git stash");
                ui::print_info("   • Then retry view creation");
            } else {
                ui::print_error(&format!("❌ Failed to checkout branch '{branch_name}'"));
                ui::print_info("🔧 Branch checkout failed:");
                ui::print_info(&format!("   • Error: {}", stderr.trim()));
                ui::print_info(&format!(
                    "   • Check branch status: cd {} && git status",
                    repo_path.display()
                ));
            }
            anyhow::bail!("Failed to checkout branch '{}': {}", branch_name, stderr);
        }
        ui::print_info(&format!(
            "    ✓ Checked out existing branch '{branch_name}'"
        ));
    } else {
        // Create new branch from current default branch
        let output = std::process::Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(repo_path)
            .output()
            .context("Failed to create new branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stderr.contains("already exists") {
                ui::print_error(&format!("❌ Branch '{branch_name}' already exists"));
                ui::print_info("🌿 Branch conflict:");
                ui::print_info(&format!(
                    "   • Use existing branch: git checkout {branch_name}"
                ));
                ui::print_info(&format!(
                    "   • Or delete existing: git branch -D {branch_name}"
                ));
                ui::print_info("   • Then retry view creation");
            } else {
                ui::print_error(&format!("❌ Failed to create branch '{branch_name}'"));
                ui::print_info("🔧 Branch creation failed:");
                ui::print_info(&format!("   • Error: {}", stderr.trim()));
                ui::print_info(&format!(
                    "   • Check repository state: cd {} && git status",
                    repo_path.display()
                ));
            }
            anyhow::bail!("Failed to create branch '{}': {}", branch_name, stderr);
        }
        ui::print_info(&format!(
            "    ✓ Created and checked out new branch '{branch_name}'"
        ));
    }

    Ok(())
}
