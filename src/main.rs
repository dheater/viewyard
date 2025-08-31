use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod config;
mod git;
mod models;
mod ui;

use commands::{onboard, view, workspace};

#[derive(Parser)]
#[command(name = "viewyard")]
#[command(about = "Multi-repository workspace management tool")]
#[command(long_about = "The refreshingly unoptimized alternative to monorepos.\n\nA clean, simple workspace for coordinated development across multiple repositories using task-based views and viewsets.")]
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
    /// Workspace operations (run from within a view)
    #[command(name = "workspace")]
    Workspace {
        #[command(subcommand)]
        action: workspace::WorkspaceCommand,
    },
    /// Interactive onboarding for new users
    Onboard,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::View { action } => view::handle_command(action),
        Commands::Workspace { action } => workspace::handle_command(action),
        Commands::Onboard => onboard::handle_command(),
    }
}
