use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Common test utilities to reduce code duplication
pub struct GitRepoSetup {
    pub seed_dir: TempDir,
    pub remote_path_str: String,
    _remote_dir: TempDir, // Keep alive but don't expose
}

impl GitRepoSetup {
    /// Creates a complete git repository setup with:
    /// - A bare remote repository
    /// - A seed repository with initial commit on main branch
    /// - Optionally a feature branch with commits
    pub fn new() -> Self {
        let remote_dir = TempDir::new().unwrap();
        let remote_path = remote_dir.path();

        // Create bare remote repository
        let output = StdCommand::new("git")
            .args(["init", "--bare"])
            .current_dir(remote_path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git init --bare failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Create seed repository
        let seed_dir = TempDir::new().unwrap();
        let seed_path = seed_dir.path();
        let remote_path_str = remote_path.to_str().unwrap().to_string();

        // Initialize seed repo
        StdCommand::new("git")
            .args(["init"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["remote", "add", "origin", &remote_path_str])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        // Create initial commit on main branch
        fs::write(seed_path.join("README.md"), "# Test Repository\n").unwrap();
        StdCommand::new("git")
            .args(["add", "README.md"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["push", "-u", "origin", "main"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        // Set remote HEAD
        StdCommand::new("git")
            .args(["symbolic-ref", "HEAD", "refs/heads/main"])
            .current_dir(remote_path)
            .output()
            .unwrap();

        Self {
            seed_dir,
            remote_path_str,
            _remote_dir: remote_dir,
        }
    }

    /// Creates a feature branch with a commit
    pub fn create_feature_branch(&self, branch_name: &str) -> &Self {
        let seed_path = self.seed_dir.path();

        StdCommand::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(seed_path)
            .output()
            .unwrap();

        fs::write(seed_path.join("feature.txt"), "feature content\n").unwrap();
        StdCommand::new("git")
            .args(["add", "feature.txt"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["commit", "-m", "Add feature"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["push", "-u", "origin", branch_name])
            .current_dir(seed_path)
            .output()
            .unwrap();

        self
    }

    /// Adds additional commits to the current branch (for upstream changes simulation)
    pub fn add_upstream_commits(&self) -> &Self {
        let seed_path = self.seed_dir.path();

        fs::write(seed_path.join("upstream.txt"), "upstream change\n").unwrap();
        StdCommand::new("git")
            .args(["add", "upstream.txt"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["commit", "-m", "Upstream change"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        StdCommand::new("git")
            .args(["push"])
            .current_dir(seed_path)
            .output()
            .unwrap();

        self
    }

    pub fn remote_url(&self) -> &str {
        &self.remote_path_str
    }
}

/// Creates a standard viewyard repository configuration
pub fn create_viewyard_config(
    viewset_dir: &Path,
    repo_name: &str,
    repo_url: &str,
    directory_name: Option<&str>,
) {
    let mut repo_config = json!({
        "name": repo_name,
        "url": repo_url,
        "is_private": false,
        "source": "GitHub (test-user)",
        "account": "test-user"
    });

    if let Some(dir_name) = directory_name {
        repo_config["directory_name"] = json!(dir_name);
    }

    let repos = json!([repo_config]);
    fs::write(
        viewset_dir.join(".viewyard-repos.json"),
        serde_json::to_string_pretty(&repos).unwrap(),
    )
    .unwrap();
}
