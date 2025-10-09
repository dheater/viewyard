use crate::ui;
use anyhow::Result;

/// Handle git clone errors with specific recovery guidance
pub fn handle_clone_error(repo_name: &str, stderr: &str) -> Result<()> {
    if stderr.contains("Permission denied") || stderr.contains("publickey") {
        show_ssh_auth_error(repo_name)
    } else if stderr.contains("not found") || stderr.contains("does not exist") {
        show_repo_not_found_error(repo_name)
    } else if stderr.contains("timeout") || stderr.contains("network") {
        show_network_error(repo_name)
    } else if stderr.contains("already exists") {
        show_directory_exists_error(repo_name)
    } else {
        show_generic_clone_error(repo_name, stderr)
    }
}

/// Handle git branch checkout errors
pub fn handle_checkout_error(
    branch_name: &str,
    repo_path: &std::path::Path,
    stderr: &str,
) -> Result<()> {
    if stderr.contains("uncommitted changes") || stderr.contains("would be overwritten") {
        show_uncommitted_changes_error(branch_name, repo_path)
    } else {
        show_generic_checkout_error(branch_name, repo_path, stderr)
    }
}

/// Handle git branch creation errors
pub fn handle_branch_creation_error(
    branch_name: &str,
    repo_path: &std::path::Path,
    stderr: &str,
) -> Result<()> {
    if stderr.contains("already exists") {
        show_branch_exists_error(branch_name)
    } else {
        show_generic_branch_creation_error(branch_name, repo_path, stderr)
    }
}

fn show_ssh_auth_error(repo_name: &str) -> Result<()> {
    ui::print_error(&format!("SSH authentication failed for {repo_name}"));
    ui::print_info("SSH key issues detected:");
    ui::print_info("   • Test SSH connection: ssh -T git@github.com");
    ui::print_info("   • Add SSH key to GitHub: gh auth refresh -h github.com -s admin:public_key");
    ui::print_info("   • Or use HTTPS: git config --global url.\"https://github.com/\".insteadOf git@github.com:");
    anyhow::bail!("SSH authentication failed for repository '{repo_name}'")
}

fn show_repo_not_found_error(repo_name: &str) -> Result<()> {
    ui::print_error(&format!("Repository not found: {repo_name}"));
    ui::print_info("Repository access issues:");
    ui::print_info(&format!(
        "   • Verify repository exists: gh repo view {repo_name}"
    ));
    ui::print_info("   • Check repository URL in .viewyard-repos.json");
    ui::print_info("   • Ensure you have access to this repository");
    anyhow::bail!("Repository '{repo_name}' not found or inaccessible")
}

fn show_network_error(repo_name: &str) -> Result<()> {
    ui::print_error(&format!("Network timeout cloning {repo_name}"));
    ui::print_info("Network issues detected:");
    ui::print_info("   • Check internet connection");
    ui::print_info("   • Try again in a few moments");
    ui::print_info("   • Consider using a VPN if behind corporate firewall");
    anyhow::bail!("Network timeout cloning repository '{repo_name}'")
}

fn show_directory_exists_error(repo_name: &str) -> Result<()> {
    ui::print_error(&format!("Directory already exists: {repo_name}"));
    ui::print_info("Directory conflict:");
    ui::print_info(&format!(
        "   • Remove existing directory: rm -rf {repo_name}"
    ));
    ui::print_info("   • Or choose a different view name");
    anyhow::bail!("Directory '{repo_name}' already exists")
}

fn show_generic_clone_error(repo_name: &str, stderr: &str) -> Result<()> {
    ui::print_error(&format!("Failed to clone {repo_name}"));
    ui::print_info("Git clone failed:");
    ui::print_info(&format!("   • Error: {}", stderr.trim()));
    ui::print_info("   • Check repository URL and permissions");
    ui::print_info("   • Verify git and network connectivity");
    anyhow::bail!("Failed to clone repository '{repo_name}': {stderr}")
}

fn show_uncommitted_changes_error(branch_name: &str, repo_path: &std::path::Path) -> Result<()> {
    ui::print_error(&format!(
        "Cannot checkout branch '{branch_name}' - uncommitted changes"
    ));
    ui::print_info("Uncommitted changes detected:");
    ui::print_info(&format!("   • Navigate to: cd {}", repo_path.display()));
    ui::print_info("   • Commit changes: git add . && git commit -m \"Save work\"");
    ui::print_info("   • Or stash changes: git stash");
    ui::print_info("   • Then retry view creation");
    anyhow::bail!("Failed to checkout branch '{branch_name}' - uncommitted changes")
}

fn show_generic_checkout_error(
    branch_name: &str,
    repo_path: &std::path::Path,
    stderr: &str,
) -> Result<()> {
    ui::print_error(&format!("Failed to checkout branch '{branch_name}'"));
    ui::print_info("Branch checkout failed:");
    ui::print_info(&format!("   • Error: {}", stderr.trim()));
    ui::print_info(&format!(
        "   • Check branch status: cd {} && git status",
        repo_path.display()
    ));
    anyhow::bail!("Failed to checkout branch '{branch_name}': {stderr}")
}

fn show_branch_exists_error(branch_name: &str) -> Result<()> {
    ui::print_error(&format!("Branch '{branch_name}' already exists"));
    ui::print_info("Branch conflict:");
    ui::print_info(&format!(
        "   • Use existing branch: git checkout {branch_name}"
    ));
    ui::print_info(&format!(
        "   • Or delete existing: git branch -D {branch_name}"
    ));
    ui::print_info("   • Then retry view creation");
    anyhow::bail!("Branch '{branch_name}' already exists")
}

fn show_generic_branch_creation_error(
    branch_name: &str,
    repo_path: &std::path::Path,
    stderr: &str,
) -> Result<()> {
    ui::print_error(&format!("Failed to create branch '{branch_name}'"));
    ui::print_info("Branch creation failed:");
    ui::print_info(&format!("   • Error: {}", stderr.trim()));
    ui::print_info(&format!(
        "   • Check repository state: cd {} && git status",
        repo_path.display()
    ));
    anyhow::bail!("Failed to create branch '{branch_name}': {stderr}")
}
