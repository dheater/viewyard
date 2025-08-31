use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewset {
    pub repos: Vec<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewsetsConfig {
    pub viewsets: HashMap<String, Viewset>,
}







// Additional models for onboarding functionality
#[derive(Debug, Clone)]
pub struct RepositorySearchResult {
    pub query: String,
    pub results: Vec<Repository>,
    pub total_available: usize,
}

impl RepositorySearchResult {
    #[must_use]
    pub const fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    #[must_use]
    pub fn is_exact_match(&self) -> bool {
        self.results.len() == 1 && self.results[0].name.to_lowercase() == self.query.to_lowercase()
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct DirectoryAnalysis {
    pub path: PathBuf,
    pub exists: bool,
    pub is_empty: bool,
    pub has_git_repos: bool,
    pub existing_repos: Vec<String>,
    pub safe_to_proceed: bool,
    pub message: String,
}

impl DirectoryAnalysis {
    #[must_use]
    pub const fn needs_confirmation(&self) -> bool {
        !self.safe_to_proceed
    }

    #[must_use]
    pub const fn can_import_repos(&self) -> bool {
        self.has_git_repos && !self.existing_repos.is_empty()
    }
}



impl Repository {
    #[must_use]
    pub const fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

impl ViewsetsConfig {
    #[must_use]
    pub fn new() -> Self {
        Self {
            viewsets: HashMap::new(),
        }
    }

    pub fn add_viewset(&mut self, name: String, viewset: Viewset) {
        self.viewsets.insert(name, viewset);
    }

    #[must_use]
    pub fn get_viewset(&self, name: &str) -> Option<&Viewset> {
        self.viewsets.get(name)
    }

    #[must_use]
    pub fn get_first_viewset_name(&self) -> Option<String> {
        self.viewsets.keys().next().cloned()
    }
}

impl Default for ViewsetsConfig {
    fn default() -> Self {
        Self::new()
    }
}

// Simple template for repository selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewTemplate {
    pub repos: Vec<String>,
}

impl ViewTemplate {
    pub fn load(template_name: &str) -> Result<Self, std::io::Error> {
        // Try viewset-specific template first
        if let Some(viewset) = crate::config::detect_viewset_for_creation() {
            if let Ok(template) = Self::load_for_viewset(template_name, &viewset) {
                return Ok(template);
            }
        }

        // Fallback to global template
        Self::load_global(template_name)
    }

    pub fn load_for_viewset(template_name: &str, viewset: &str) -> Result<Self, std::io::Error> {
        let template_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("viewyard")
            .join("templates")
            .join(viewset)
            .join(format!("{template_name}.yaml"));

        let content = std::fs::read_to_string(&template_path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn load_global(template_name: &str) -> Result<Self, std::io::Error> {
        let template_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("viewyard")
            .join("templates")
            .join(format!("{template_name}.yaml"));

        let content = std::fs::read_to_string(&template_path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    #[must_use]
    pub fn template_exists(template_name: &str) -> bool {
        // Check viewset-specific template first
        if let Some(viewset) = crate::config::detect_viewset_for_creation() {
            if Self::template_exists_for_viewset(template_name, &viewset) {
                return true;
            }
        }

        // Check global template
        Self::template_exists_global(template_name)
    }

    #[must_use]
    pub fn template_exists_for_viewset(template_name: &str, viewset: &str) -> bool {
        let template_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("viewyard")
            .join("templates")
            .join(viewset)
            .join(format!("{template_name}.yaml"));

        template_path.exists()
    }

    #[must_use]
    pub fn template_exists_global(template_name: &str) -> bool {
        let template_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("viewyard")
            .join("templates")
            .join(format!("{template_name}.yaml"));

        template_path.exists()
    }

    #[must_use]
    pub fn list_available() -> Vec<String> {
        let mut templates = Vec::new();

        // Add viewset-specific templates first
        if let Some(viewset) = crate::config::detect_viewset_for_creation() {
            let viewset_templates = Self::list_available_for_viewset(&viewset);
            for template in viewset_templates {
                templates.push(format!("{template} ({viewset})"));
            }
        }

        // Add global templates
        let global_templates = Self::list_available_global();
        for template in global_templates {
            templates.push(format!("{template} (global)"));
        }

        templates
    }

    /// # Panics
    /// Panics if the current directory cannot be read when the config directory is not available.
    #[must_use]
    pub fn list_available_for_viewset(viewset: &str) -> Vec<String> {
        let templates_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("viewyard")
            .join("templates")
            .join(viewset);

        if !templates_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(&templates_dir)
            .unwrap_or_else(|_| std::fs::read_dir(".").unwrap())
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "yaml" {
                    path.file_stem()?.to_str().map(std::string::ToString::to_string)
                } else {
                    None
                }
            })
            .collect()
    }

    /// # Panics
    /// Panics if the current directory cannot be read when the config directory is not available.
    #[must_use]
    pub fn list_available_global() -> Vec<String> {
        let templates_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("viewyard")
            .join("templates");

        if !templates_dir.exists() {
            return Vec::new();
        }

        std::fs::read_dir(&templates_dir)
            .unwrap_or_else(|_| std::fs::read_dir(".").unwrap())
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("yaml")) {
                    path.file_stem()?.to_str().map(std::string::ToString::to_string)
                } else {
                    None
                }
            })
            .collect()
    }
}
