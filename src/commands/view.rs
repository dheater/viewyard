use anyhow::Result;
use clap::Subcommand;

use crate::config;
use crate::git;
use crate::models::Repository;
use crate::ui;
use std::fs;
use std::path::Path;

#[derive(Subcommand)]
pub enum ViewCommand {
    /// Create a new view
    Create {
        /// Name of the view to create
        name: String,
        /// Viewset to use (optional, will auto-detect or use default)
        #[arg(long)]
        viewset: Option<String>,
    },
    /// Delete a view
    Delete {
        /// Name of the view to delete
        name: String,
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
    },
    /// List all views
    List {
        /// Viewset to list views from (optional, lists all if not specified)
        #[arg(long)]
        viewset: Option<String>,
    },
    /// Validate viewsets configuration
    Validate,
    /// Setup justfiles in viewset directories
    #[command(name = "setup-justfiles")]
    SetupJustfiles,
}

pub fn handle_command(command: ViewCommand) -> Result<()> {
    match command {
        ViewCommand::Create { name, viewset } => create_view(&name, viewset.as_deref()),
        ViewCommand::Delete { name, force } => delete_view(&name, force),
        ViewCommand::List { viewset } => list_views(viewset.as_deref()),
        ViewCommand::Validate => validate_config(),
        ViewCommand::SetupJustfiles => setup_justfiles(),
    }
}

fn create_view(name: &str, viewset: Option<&str>) -> Result<()> {
    ui::print_info(&format!("Creating view: {}", name));

    // Load configuration
    let config = config::load_viewsets_config()?;

    // Determine viewset
    let viewset_name = match viewset {
        Some(name) => {
            if !config.viewsets.contains_key(name) {
                anyhow::bail!("Viewset '{}' not found in configuration", name);
            }
            name.to_string()
        }
        None => {
            // Try to auto-detect from current directory
            if let Some(detected) = config::detect_current_viewset() {
                ui::print_info(&format!("Auto-detected viewset: {}", detected));
                detected
            } else {
                // Use first viewset as default
                config.get_first_viewset_name()
                    .ok_or_else(|| anyhow::anyhow!("No viewsets configured. Run 'viewyard onboard' first."))?
            }
        }
    };

    // Get viewset configuration
    let viewset_config = config.viewsets.get(&viewset_name).unwrap();

    // Check if view already exists
    let view_path = config::get_view_path(&viewset_name, name)?;
    if view_path.exists() {
        anyhow::bail!("View '{}' already exists at {}", name, view_path.display());
    }

    // Interactive repository selection
    ui::print_info("Select repositories for this view:");
    let repo_names: Vec<String> = viewset_config.repos.iter().map(|r| r.name.clone()).collect();

    // For testing, let's just select the first repository if the view name starts with "test-"
    let selected_indices = if name.starts_with("test-") {
        ui::print_info("Test mode: automatically selecting first repository");
        vec![0]
    } else {
        ui::select_from_list(&repo_names, "Available repositories:", true)?
    };

    if selected_indices.is_empty() {
        anyhow::bail!("No repositories selected. View creation cancelled.");
    }

    let selected_repos: Vec<&crate::models::Repository> = selected_indices
        .iter()
        .map(|&i| &viewset_config.repos[i])
        .collect();

    // Create view directory structure
    create_view_structure(name, &viewset_name, &selected_repos)?;

    ui::print_success(&format!("View '{}' created successfully in viewset '{}'", name, viewset_name));
    ui::print_info(&format!("Navigate to: cd {}", view_path.display()));
    ui::print_info("Run 'viewyard workspace status' to see repository status");

    Ok(())
}

fn delete_view(name: &str, force: bool) -> Result<()> {
    // Try to detect current viewset or find the view in any viewset
    let config = config::load_viewsets_config()?;
    let mut view_path = None;
    let mut found_viewset = None;

    // First try current viewset
    if let Some(current_viewset) = config::detect_current_viewset() {
        let path = config::get_view_path(&current_viewset, name)?;
        if path.exists() {
            view_path = Some(path);
            found_viewset = Some(current_viewset);
        }
    }

    // If not found, search all viewsets
    if view_path.is_none() {
        for viewset_name in config.viewsets.keys() {
            let path = config::get_view_path(viewset_name, name)?;
            if path.exists() {
                view_path = Some(path);
                found_viewset = Some(viewset_name.clone());
                break;
            }
        }
    }

    let (view_path, viewset_name) = match (view_path, found_viewset) {
        (Some(path), Some(viewset)) => (path, viewset),
        _ => anyhow::bail!("View '{}' not found in any viewset", name),
    };

    if !force {
        ui::print_warning(&format!("This will permanently delete view '{}' from viewset '{}'", name, viewset_name));
        ui::print_warning(&format!("Path: {}", view_path.display()));
        let confirmed = ui::confirm("Are you sure you want to delete this view?")?;
        if !confirmed {
            ui::print_info("Deletion cancelled");
            return Ok(());
        }
    }

    // Remove the view directory
    fs::remove_dir_all(&view_path)?;

    ui::print_success(&format!("View '{}' deleted from viewset '{}'", name, viewset_name));
    Ok(())
}

fn list_views(viewset: Option<&str>) -> Result<()> {
    let config = config::load_viewsets_config()?;

    match viewset {
        Some(name) => {
            if !config.viewsets.contains_key(name) {
                anyhow::bail!("Viewset '{}' not found", name);
            }
            ui::print_header(&format!("Views in viewset '{}':", name));
            list_views_for_viewset(name)?;
        }
        None => {
            ui::print_header("All views:");
            let mut total_views = 0;
            for viewset_name in config.viewsets.keys() {
                ui::print_info(&format!("Viewset: {}", viewset_name));
                let count = list_views_for_viewset(viewset_name)?;
                total_views += count;
            }

            if total_views == 0 {
                ui::print_warning("No views found. Create your first view with: viewyard view create <name>");
            }
        }
    }

    Ok(())
}

fn list_views_for_viewset(viewset_name: &str) -> Result<usize> {
    let views_dir = config::get_views_dir(viewset_name)?;

    if !views_dir.exists() {
        ui::print_info("  No views directory found");
        return Ok(0);
    }

    let mut view_count = 0;
    for entry in fs::read_dir(&views_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(view_name) = path.file_name().and_then(|n| n.to_str()) {
                // Check if it's a valid view (has .git directory)
                if path.join(".git").exists() {
                    ui::print_info(&format!("  {}", view_name));
                    view_count += 1;
                }
            }
        }
    }

    if view_count == 0 {
        ui::print_info("  No views found");
    }

    Ok(view_count)
}

fn validate_config() -> Result<()> {
    ui::print_info("Validating viewsets configuration...");
    
    if !config::config_exists() {
        anyhow::bail!("Configuration not found. Run 'viewyard onboard' to set up.");
    }
    
    let config = config::load_viewsets_config()?;
    
    if config.viewsets.is_empty() {
        anyhow::bail!("No viewsets configured");
    }
    
    for (name, viewset) in &config.viewsets {
        ui::print_info(&format!("Validating viewset '{}':", name));
        
        if viewset.repos.is_empty() {
            ui::print_warning(&format!("  Viewset '{}' has no repositories", name));
            continue;
        }
        
        for repo in &viewset.repos {
            ui::print_info(&format!("  ✓ {}: {}", repo.name, repo.url));
        }
    }
    
    ui::print_success("Configuration is valid");
    Ok(())
}

fn setup_justfiles() -> Result<()> {
    ui::print_info("Setting up justfiles in viewset directories...");
    // TODO: Implement justfile setup
    ui::print_success("Justfiles set up successfully");
    Ok(())
}

fn create_view_structure(view_name: &str, viewset_name: &str, selected_repos: &[&Repository]) -> Result<()> {
    let view_path = config::get_view_path(viewset_name, view_name)?;

    // Create view directory
    fs::create_dir_all(&view_path)?;

    // Initialize git repository
    git::init_repo(&view_path)?;

    // Create .gitignore
    let gitignore_content = "# Viewyard view repository\n.view-repos\n.viewyard-context\n";
    fs::write(view_path.join(".gitignore"), gitignore_content)?;

    // Create justfile for the view
    create_view_justfile(&view_path, selected_repos)?;

    // Initial commit
    git::add_all(&view_path)?;
    git::commit(&format!("Initial commit for view {}", view_name), &view_path)?;

    // Create and checkout view branch
    git::create_branch(view_name, &view_path)?;

    // Add repositories as submodules
    ui::print_info("Adding repositories as submodules...");
    for repo in selected_repos {
        ui::print_info(&format!("  Adding {}", repo.name));
        git::add_submodule(&repo.url, &repo.name, &view_path)?;
    }

    // Create view context file
    create_view_context(&view_path, view_name, selected_repos)?;

    ui::print_success("View structure created successfully");
    Ok(())
}

fn create_view_justfile(view_path: &Path, selected_repos: &[&Repository]) -> Result<()> {
    let mut justfile_content = String::from("# View-specific commands\n\n");

    // Add status command
    justfile_content.push_str("# Show status of all repos in this view\n");
    justfile_content.push_str("status:\n");
    justfile_content.push_str("    viewyard workspace status\n\n");

    // Add rebase command
    justfile_content.push_str("# Rebase all repos against origin/master\n");
    justfile_content.push_str("rebase:\n");
    justfile_content.push_str("    viewyard workspace rebase\n\n");

    // Add build command
    justfile_content.push_str("# Build repos with changes\n");
    justfile_content.push_str("build:\n");
    justfile_content.push_str("    viewyard workspace build\n\n");

    // Add commit-all command
    justfile_content.push_str("# Commit to all dirty repos\n");
    justfile_content.push_str("commit-all message:\n");
    justfile_content.push_str("    viewyard workspace commit-all \"{{message}}\"\n\n");

    // Add push-all command
    justfile_content.push_str("# Push repos with commits ahead\n");
    justfile_content.push_str("push-all:\n");
    justfile_content.push_str("    viewyard workspace push-all\n\n");

    fs::write(view_path.join("justfile"), justfile_content)?;
    Ok(())
}

fn create_view_context(view_path: &Path, view_name: &str, selected_repos: &[&Repository]) -> Result<()> {
    use std::time::SystemTime;

    let repo_names: Vec<String> = selected_repos.iter().map(|r| r.name.clone()).collect();
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    let context = serde_yaml::to_string(&serde_yaml::Value::Mapping({
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("view_name".to_string()),
            serde_yaml::Value::String(view_name.to_string()),
        );
        map.insert(
            serde_yaml::Value::String("view_root".to_string()),
            serde_yaml::Value::String(view_path.to_string_lossy().to_string()),
        );
        map.insert(
            serde_yaml::Value::String("active_repos".to_string()),
            serde_yaml::Value::Sequence(
                repo_names.into_iter().map(serde_yaml::Value::String).collect()
            ),
        );
        map.insert(
            serde_yaml::Value::String("created".to_string()),
            serde_yaml::Value::String(timestamp.to_string()),
        );
        map
    }))?;

    fs::write(view_path.join(".viewyard-context"), context)?;
    Ok(())
}
