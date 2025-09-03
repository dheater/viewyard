use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub is_private: bool,
    pub source: String, // e.g., "GitHub (username)" or "GitHub (org/username)"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>, // Optional explicit account field for git user configuration
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
