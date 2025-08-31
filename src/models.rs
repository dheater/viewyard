use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewset {
    pub repos: Vec<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewsetsConfig {
    pub viewsets: HashMap<String, Viewset>,
}

#[derive(Debug, Clone)]
pub struct GitContext {
    pub name: String,
    pub user_name: String,
    pub user_email: String,
    pub ssh_key_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ViewContext {
    pub view_name: String,
    pub view_root: PathBuf,
    pub active_repos: Vec<String>,
    pub created: String,
}

#[derive(Debug, Clone)]
pub struct OnboardingState {
    pub git_contexts: Vec<GitContext>,
    pub discovered_repositories: Vec<Repository>,
    pub existing_viewsets: HashMap<String, Viewset>,
    pub contexts_to_setup: Vec<String>,
}

impl Repository {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            build: None,
            test: None,
        }
    }

    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }

    pub fn with_test(mut self, test: String) -> Self {
        self.test = Some(test);
        self
    }
}

impl ViewsetsConfig {
    pub fn new() -> Self {
        Self {
            viewsets: HashMap::new(),
        }
    }

    pub fn add_viewset(&mut self, name: String, viewset: Viewset) {
        self.viewsets.insert(name, viewset);
    }

    pub fn get_viewset(&self, name: &str) -> Option<&Viewset> {
        self.viewsets.get(name)
    }

    pub fn get_first_viewset_name(&self) -> Option<String> {
        self.viewsets.keys().next().cloned()
    }
}

impl Default for ViewsetsConfig {
    fn default() -> Self {
        Self::new()
    }
}
