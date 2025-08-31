use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::models::ViewsetsConfig;

/// Get the viewyard configuration directory
pub fn config_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home).join(".config").join("viewyard"))
}

/// Get the path to the viewsets configuration file
pub fn viewsets_config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("viewsets.yaml"))
}

/// Load viewsets configuration from ~/.config/viewyard/viewsets.yaml
pub fn load_viewsets_config() -> Result<ViewsetsConfig> {
    let config_path = viewsets_config_path()?;
    
    if !config_path.exists() {
        anyhow::bail!(
            "Viewsets configuration not found at {}\nRun 'viewyard onboard' to set up your configuration",
            config_path.display()
        );
    }

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: ViewsetsConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    Ok(config)
}

/// Save viewsets configuration to ~/.config/viewyard/viewsets.yaml
pub fn save_viewsets_config(config: &ViewsetsConfig) -> Result<()> {
    let config_path = viewsets_config_path()?;
    
    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let content = serde_yaml::to_string(config)
        .context("Failed to serialize viewsets configuration")?;

    fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    Ok(())
}

/// Check if viewsets configuration exists
pub fn config_exists() -> bool {
    viewsets_config_path()
        .map(|path| path.exists())
        .unwrap_or(false)
}

/// Get the workspace root directory for a viewset
pub fn get_viewset_root(viewset_name: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home).join("src").join(format!("src-{}", viewset_name)))
}

/// Get the views directory for a viewset
pub fn get_views_dir(viewset_name: &str) -> Result<PathBuf> {
    Ok(get_viewset_root(viewset_name)?.join("views"))
}

/// Get the path to a specific view
pub fn get_view_path(viewset_name: &str, view_name: &str) -> Result<PathBuf> {
    Ok(get_views_dir(viewset_name)?.join(view_name))
}

/// Detect current viewset from working directory
pub fn detect_current_viewset() -> Option<String> {
    let current_dir = std::env::current_dir().ok()?;
    let current_str = current_dir.to_string_lossy();
    
    // Look for pattern like ~/src/src-<viewset>/views/<view>
    if let Some(src_pos) = current_str.find("/src/src-") {
        let after_src = &current_str[src_pos + 9..]; // Skip "/src/src-"
        if let Some(slash_pos) = after_src.find('/') {
            return Some(after_src[..slash_pos].to_string());
        }
    }
    
    None
}

/// Check if we're currently in a view directory
pub fn detect_current_view() -> Option<(String, String)> {
    let current_dir = std::env::current_dir().ok()?;
    let current_str = current_dir.to_string_lossy();
    
    // Look for pattern like ~/src/src-<viewset>/views/<view>
    if let Some(src_pos) = current_str.find("/src/src-") {
        let after_src = &current_str[src_pos + 9..]; // Skip "/src/src-"
        let parts: Vec<&str> = after_src.split('/').collect();
        
        if parts.len() >= 3 && parts[1] == "views" {
            return Some((parts[0].to_string(), parts[2].to_string()));
        }
    }
    
    None
}
