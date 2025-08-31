use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

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

/// Load viewsets configuration from a specific path
pub fn load_viewsets_config_from_path(config_path: &Path) -> Result<ViewsetsConfig> {
    if !config_path.exists() {
        anyhow::bail!(
            "Viewsets configuration not found at {}",
            config_path.display()
        );
    }

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: ViewsetsConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    Ok(config)
}

/// Load viewsets configuration from ~/.config/viewyard/viewsets.yaml
pub fn load_viewsets_config() -> Result<ViewsetsConfig> {
    let config_path = viewsets_config_path()?;
    load_viewsets_config_from_path(&config_path)
}



/// Check if viewsets configuration exists
#[must_use]
pub fn config_exists() -> bool {
    viewsets_config_path()
        .map(|path| path.exists())
        .unwrap_or(false)
}

/// Get the workspace root directory for a viewset
pub fn get_viewset_root(viewset_name: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home)
        .join("src")
        .join(format!("src-{viewset_name}")))
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
/// Looks for pattern: ~/src/src-<viewset>/views/<view>
#[must_use]
pub fn detect_current_viewset() -> Option<String> {
    let current_dir = std::env::current_dir().ok()?;

    // Walk up the path components looking for the pattern
    let components: Vec<_> = current_dir.components().collect();

    for (i, component) in components.iter().enumerate() {
        if let std::path::Component::Normal(name) = component {
            let name_str = name.to_string_lossy();

            // Look for "src-<viewset>" pattern
            if let Some(viewset) = name_str.strip_prefix("src-") {
                // Verify this is in the right context (preceded by "src")
                if i > 0 {
                    if let std::path::Component::Normal(prev_name) = &components[i - 1] {
                        if prev_name.to_string_lossy() == "src" {
                            return Some(viewset.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}



/// Detect viewset for view creation
/// More flexible than `detect_current_viewset` - works from ~/src/src-<viewset>/ and subdirectories
#[must_use]
pub fn detect_viewset_for_creation() -> Option<String> {
    // This is actually the same logic as detect_current_viewset now that we use proper path traversal
    // The "flexibility" was just due to the fragile string parsing before
    detect_current_viewset()
}
