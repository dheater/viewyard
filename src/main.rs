use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

mod commands;
mod config;
mod git;
mod models;
mod ui;

use commands::{view, workspace};
use models::ViewsetsConfig;

#[derive(Parser)]
#[command(name = "viewyard")]
#[command(about = "Multi-repository workspace management tool")]
#[command(
    long_about = "The refreshingly unoptimized alternative to monorepos.\n\nA clean, simple workspace for coordinated development across multiple repositories using task-based views and viewsets."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View management commands
    View {
        #[command(subcommand)]
        action: view::ViewCommand,
    },

    // Workspace commands (top-level for convenience)
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
    /// Switch to a view (works from anywhere)
    Switch {
        /// Name of the view to switch to
        view_name: String,
    },
    /// List views (context-aware)
    Views,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::View { action } => view::handle_command(action),

        // Workspace commands
        Commands::Status => workspace::handle_command(workspace::WorkspaceCommand::Status),
        Commands::Rebase => workspace::handle_command(workspace::WorkspaceCommand::Rebase),
        Commands::CommitAll { message } => {
            workspace::handle_command(workspace::WorkspaceCommand::CommitAll { message })
        }
        Commands::PushAll => workspace::handle_command(workspace::WorkspaceCommand::PushAll),
        Commands::Switch { view_name } => switch_to_view(&view_name),
        Commands::Views => list_views_context_aware(),
    }
}

fn switch_to_view(view_name: &str) -> Result<()> {
    // Load configuration
    let config = config::load_viewsets_config()?;

    // Find the view across all viewsets
    let mut found_view: Option<(String, String)> = None;

    for (viewset_name, _viewset) in &config.viewsets {
        let view_path = match config::get_view_path(viewset_name, view_name) {
            Ok(path) => path,
            Err(_) => continue,
        };

        if view_path.exists() {
            found_view = Some((
                viewset_name.clone(),
                view_path.to_string_lossy().to_string(),
            ));
            break;
        }
    }

    match found_view {
        Some((viewset_name, view_path)) => {
            ui::print_info(&format!(
                "🎯 Switching to view '{}' in viewset '{}'",
                view_name, viewset_name
            ));
            ui::print_info(&format!("📂 Path: {}", view_path));

            // Change to the view directory
            std::env::set_current_dir(&view_path)?;

            // Launch a new shell in the view directory
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

            ui::print_success("🚀 Launching shell in view directory...");
            ui::print_info("💡 Use 'exit' to return to your previous directory");

            let status = Command::new(&shell).current_dir(&view_path).status()?;

            if !status.success() {
                ui::print_warning("Shell exited with non-zero status");
            }

            Ok(())
        }
        None => {
            ui::show_error_with_help(
                &format!("View '{}' not found", view_name),
                &[
                    "Available views:",
                    &list_all_views(&config),
                    "",
                    "Create a new view with: viewyard view create <name>",
                ],
            );
            Err(anyhow::anyhow!("View not found"))
        }
    }
}

fn list_views_context_aware() -> Result<()> {
    let config = config::load_viewsets_config()?;

    // Try to detect current viewset
    if let Some(current_viewset) = config::detect_viewset_for_creation() {
        if config.viewsets.contains_key(&current_viewset) {
            ui::print_header(&format!("Views in viewset '{}'", current_viewset));
            list_views_for_viewset(&config, &current_viewset)?;

            // Show other viewsets if they exist
            let other_viewsets: Vec<_> = config
                .viewsets
                .keys()
                .filter(|&k| k != &current_viewset)
                .collect();

            if !other_viewsets.is_empty() {
                ui::print_info("");
                ui::print_info("💡 Other viewsets:");
                for viewset_name in other_viewsets {
                    ui::print_info(&format!("   viewyard view list --viewset {}", viewset_name));
                }
            }

            return Ok(());
        }
    }

    // Fallback: show all views grouped by viewset
    ui::print_header("All Views");

    let mut has_views = false;
    for (viewset_name, _viewset) in &config.viewsets {
        if let Ok(views_dir) = config::get_views_dir(viewset_name) {
            if views_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&views_dir) {
                    let view_names: Vec<_> = entries
                        .flatten()
                        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                        .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
                        .collect();

                    if !view_names.is_empty() {
                        if has_views {
                            ui::print_info("");
                        }
                        ui::print_info(&format!(
                            "📂 {} ({} views):",
                            viewset_name,
                            view_names.len()
                        ));
                        for view_name in view_names {
                            ui::print_info(&format!("   • {}", view_name));
                        }
                        has_views = true;
                    }
                }
            }
        }
    }

    if !has_views {
        ui::print_info("No views found");
        ui::print_info("💡 Create your first view with: viewyard view create <name>");
    }

    Ok(())
}

fn list_views_for_viewset(_config: &ViewsetsConfig, viewset_name: &str) -> Result<()> {
    if let Ok(views_dir) = config::get_views_dir(viewset_name) {
        if views_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&views_dir) {
                let view_names: Vec<_> = entries
                    .flatten()
                    .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                    .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
                    .collect();

                if view_names.is_empty() {
                    ui::print_info("No views found in this viewset");
                    ui::print_info(&format!(
                        "💡 Create a view with: viewyard view create <name> --viewset {}",
                        viewset_name
                    ));
                } else {
                    for view_name in view_names {
                        ui::print_info(&format!("  • {}", view_name));
                    }
                }
            }
        } else {
            ui::print_info("Views directory doesn't exist yet");
            ui::print_info(&format!(
                "💡 Create a view with: viewyard view create <name> --viewset {}",
                viewset_name
            ));
        }
    }

    Ok(())
}

fn list_all_views(config: &ViewsetsConfig) -> String {
    let mut views = Vec::new();

    for (viewset_name, _viewset) in &config.viewsets {
        if let Ok(views_dir) = config::get_views_dir(viewset_name) {
            if views_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&views_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            if let Some(view_name) = entry.file_name().to_str() {
                                views.push(format!("  • {} ({})", view_name, viewset_name));
                            }
                        }
                    }
                }
            }
        }
    }

    if views.is_empty() {
        "  No views found".to_string()
    } else {
        views.join("\n")
    }
}
