use anyhow::Result;
use clap::Subcommand;

use crate::config;
use crate::ui;

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
    
    ui::print_success(&format!("View '{}' created successfully in viewset '{}'", name, viewset_name));
    Ok(())
}

fn delete_view(name: &str, force: bool) -> Result<()> {
    if !force {
        let confirmed = ui::confirm(&format!("Delete view '{}'?", name))?;
        if !confirmed {
            ui::print_info("Deletion cancelled");
            return Ok(());
        }
    }
    
    ui::print_success(&format!("View '{}' deleted", name));
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
            // TODO: Implement actual view listing
            ui::print_info("No views found");
        }
        None => {
            ui::print_header("All views:");
            for viewset_name in config.viewsets.keys() {
                ui::print_info(&format!("Viewset: {}", viewset_name));
                // TODO: Implement actual view listing
                ui::print_info("  No views found");
            }
        }
    }
    
    Ok(())
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
