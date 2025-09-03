use anyhow::{Context, Result};
use std::process::Command;

use crate::models::Repository;

#[derive(Debug)]
pub struct GitHubService;

impl GitHubService {
    /// Check if GitHub CLI is available and authenticated
    pub fn check_availability() -> Result<bool> {
        let output = Command::new("gh")
            .args(["--version"])
            .output()
            .context("Failed to check if gh CLI is installed")?;

        if !output.status.success() {
            return Ok(false);
        }

        // Check if authenticated
        let auth_output = Command::new("gh")
            .args(["auth", "status"])
            .output()
            .context("Failed to check gh CLI authentication status")?;

        Ok(auth_output.status.success())
    }

    /// Get list of available GitHub accounts
    pub fn get_available_accounts() -> Result<Vec<String>> {
        let output = Command::new("gh")
            .args(["auth", "status"])
            .output()
            .context("Failed to get GitHub auth status")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut accounts = Vec::new();

        for line in stdout.lines() {
            if line.contains("✓ Logged in to github.com account") {
                if let Some(account_part) = line.split("account ").nth(1) {
                    if let Some(account) = account_part.split(' ').next() {
                        let account = account.trim();
                        if !account.is_empty() {
                            accounts.push(account.to_string());
                        }
                    }
                }
            }
        }

        Ok(accounts)
    }

    /// Get current authenticated account
    pub fn get_current_account() -> Result<String> {
        let output = Command::new("gh")
            .args(["api", "user", "--jq", ".login"])
            .output()
            .context("Failed to get current GitHub account")?;

        if !output.status.success() {
            anyhow::bail!("Failed to get current account");
        }

        let account = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(account)
    }

    /// Switch to a specific GitHub account
    pub fn switch_account(account: &str) -> Result<()> {
        let output = Command::new("gh")
            .args(["auth", "switch", "--user", account])
            .output()
            .context("Failed to switch GitHub account")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to switch to account '{}': {}", account, stderr);
        }

        Ok(())
    }

    /// Discover repositories from a specific GitHub account
    pub fn discover_repositories_from_account(account: &str) -> Result<Vec<Repository>> {
        use crate::ui;

        let mut repos = Vec::new();

        // Switch to the specified account first
        ui::print_info(&format!("  Switching to account: {account}"));
        Self::switch_account(account)?;

        // Get user repositories
        ui::print_info(&format!("  Fetching personal repositories for {account}"));
        let user_repos = Self::get_user_repositories(account)?;
        ui::print_info(&format!(
            "    Found {} personal repositories",
            user_repos.len()
        ));
        repos.extend(user_repos);

        // Get organization repositories
        ui::print_info(&format!(
            "  Fetching organization repositories for {account}"
        ));
        let org_repos = Self::get_organization_repositories(account)?;
        ui::print_info(&format!(
            "    Found {} organization repositories",
            org_repos.len()
        ));
        repos.extend(org_repos);

        ui::print_info(&format!(
            "  Total repositories for {}: {}",
            account,
            repos.len()
        ));
        Ok(repos)
    }

    /// Get user's personal repositories
    fn get_user_repositories(account: &str) -> Result<Vec<Repository>> {
        let output = Command::new("gh")
            .args([
                "repo",
                "list",
                "--limit",
                "1000",
                "--json",
                "name,sshUrl,isPrivate",
            ])
            .output()
            .context("Failed to get user repositories")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stderr.contains("authentication") || stderr.contains("not authenticated") {
                use crate::ui;
                ui::print_error("❌ GitHub CLI authentication failed");
                ui::print_info("🔑 Authentication issues:");
                ui::print_info("   • Re-authenticate: gh auth login");
                ui::print_info("   • Check auth status: gh auth status");
                ui::print_info("   • Refresh token: gh auth refresh");
                anyhow::bail!("GitHub CLI authentication failed");
            } else if stderr.contains("rate limit") || stderr.contains("API rate limit") {
                use crate::ui;
                ui::print_error("❌ GitHub API rate limit exceeded");
                ui::print_info("⏰ Rate limit issues:");
                ui::print_info("   • Wait for rate limit reset (usually 1 hour)");
                ui::print_info("   • Check rate limit: gh api rate_limit");
                ui::print_info("   • Use personal access token for higher limits");
                anyhow::bail!("GitHub API rate limit exceeded");
            } else if stderr.contains("network") || stderr.contains("timeout") {
                use crate::ui;
                ui::print_error("❌ Network error accessing GitHub");
                ui::print_info("🌐 Network issues:");
                ui::print_info("   • Check internet connection");
                ui::print_info("   • Try again in a few moments");
                ui::print_info("   • Check GitHub status: https://www.githubstatus.com/");
                anyhow::bail!("Network error accessing GitHub");
            }
            anyhow::bail!(
                "Failed to list repositories for account '{}': {}",
                account,
                stderr
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let repos_json: Vec<serde_json::Value> =
            serde_json::from_str(&stdout).context("Failed to parse user repositories JSON")?;

        let mut repos = Vec::new();
        for repo_data in repos_json {
            if let (Some(name), Some(url), Some(is_private)) = (
                repo_data["name"].as_str(),
                repo_data["sshUrl"].as_str(),
                repo_data["isPrivate"].as_bool(),
            ) {
                let privacy_indicator = if is_private { " [private]" } else { "" };
                repos.push(Repository {
                    name: name.to_string(),
                    url: url.to_string(),
                    is_private,
                    source: format!("GitHub ({account}){privacy_indicator}"),
                    account: Some(account.to_string()),
                });
            }
        }

        // Warn if we might have hit the limit
        if repos.len() >= 1000 {
            use crate::ui;
            ui::print_warning(&format!("    Warning: Found exactly 1000 repositories for {account}. Some repositories may not be shown due to GitHub CLI limits."));
        }

        Ok(repos)
    }

    /// Get repositories from organizations the user belongs to
    fn get_organization_repositories(account: &str) -> Result<Vec<Repository>> {
        // First, get list of organizations
        let orgs_output = Command::new("gh")
            .args(["api", "user/orgs", "--jq", ".[].login"])
            .output()
            .context("Failed to get user organizations")?;

        if !orgs_output.status.success() {
            return Ok(Vec::new());
        }

        let orgs_stdout = String::from_utf8_lossy(&orgs_output.stdout);
        let orgs: Vec<&str> = orgs_stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        if !orgs.is_empty() {
            use crate::ui;
            ui::print_info(&format!("    Found {} organizations to check", orgs.len()));
        }

        let mut all_repos = Vec::new();

        for (i, org) in orgs.iter().enumerate() {
            use crate::ui;
            ui::print_info(&format!(
                "    Checking organization {} ({}/{})",
                org,
                i + 1,
                orgs.len()
            ));
            let org_repos = Self::get_repositories_for_organization(org, account)?;
            ui::print_info(&format!(
                "      Found {} repositories in {}",
                org_repos.len(),
                org
            ));
            all_repos.extend(org_repos);
        }

        Ok(all_repos)
    }

    /// Get repositories for a specific organization
    fn get_repositories_for_organization(org: &str, account: &str) -> Result<Vec<Repository>> {
        let output = Command::new("gh")
            .args([
                "repo",
                "list",
                org,
                "--limit",
                "1000",
                "--json",
                "name,sshUrl,isPrivate",
            ])
            .output()
            .context("Failed to get organization repositories")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Don't fail completely if we can't access one org - just warn and continue
            eprintln!("Warning: Failed to list repositories for organization '{org}': {stderr}");
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let repos_json: Vec<serde_json::Value> = serde_json::from_str(&stdout)
            .context("Failed to parse organization repositories JSON")?;

        let mut repos = Vec::new();
        for repo_data in repos_json {
            if let (Some(name), Some(url), Some(is_private)) = (
                repo_data["name"].as_str(),
                repo_data["sshUrl"].as_str(),
                repo_data["isPrivate"].as_bool(),
            ) {
                let privacy_indicator = if is_private { " [private]" } else { "" };
                repos.push(Repository {
                    name: name.to_string(),
                    url: url.to_string(),
                    is_private,
                    source: format!("GitHub ({org}/{account}){privacy_indicator}"),
                    account: Some(account.to_string()),
                });
            }
        }

        // Warn if we might have hit the limit
        if repos.len() >= 1000 {
            use crate::ui;
            ui::print_warning(&format!("        Warning: Found exactly 1000 repositories for organization '{org}'. Some repositories may not be shown due to GitHub CLI limits."));
        }

        Ok(repos)
    }

    /// Discover all repositories from all available accounts
    pub fn discover_all_repositories() -> Result<Vec<Repository>> {
        let accounts = Self::get_available_accounts()?;

        if accounts.is_empty() {
            anyhow::bail!("No GitHub accounts found. Please run 'gh auth login' first.");
        }

        // Remember current account to restore later
        let original_account = Self::get_current_account().ok();

        let mut all_repos = Vec::new();

        for account in &accounts {
            match Self::discover_repositories_from_account(account) {
                Ok(repos) => {
                    all_repos.extend(repos);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to discover repositories from account '{account}': {e}"
                    );
                }
            }
        }

        // Restore original account
        if let Some(original) = original_account {
            if accounts.contains(&original) {
                let _ = Self::switch_account(&original);
            }
        }

        Ok(all_repos)
    }
}
