use crate::models::Repository;
use crate::search::RepositorySearch;
use anyhow::Result;
use std::io::{self, Write};

pub struct InteractiveSelector {
    search: RepositorySearch,
}

impl InteractiveSelector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            search: RepositorySearch::new(),
        }
    }

    /// Interactive repository selection with iterative search and numbered list selection
    pub fn select_repositories(&self, repositories: &[Repository]) -> Result<Vec<Repository>> {
        if repositories.is_empty() {
            println!("No repositories found.");
            return Ok(Vec::new());
        }

        println!("🔍 Repository Selection");
        println!("Found {} repositories", repositories.len());
        println!();

        // Show grouped repositories for context
        Self::show_repository_overview(repositories);
        println!();

        let mut selected_repos: Vec<Repository> = Vec::new();

        loop {
            // Show current selection status
            if !selected_repos.is_empty() {
                println!("Currently selected: {} repositories", selected_repos.len());
                for repo in &selected_repos {
                    println!("  ✓ {}", repo.name);
                }
                println!();
            }

            // Get search query
            print!("Search repositories (or 'done' to finish): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let query = input.trim();

            if query.is_empty() || query == "done" || query == "quit" {
                break;
            }

            // Find matching repositories
            let repos_to_show = if query == "all" {
                repositories.to_vec()
            } else {
                let matches = self.search.search(repositories, query);
                matches.into_iter().map(|(repo, _score)| repo).collect()
            };

            if repos_to_show.is_empty() {
                println!("No repositories found matching '{query}'");
                continue;
            }

            // Filter out already selected repositories
            let available_repos: Vec<Repository> = repos_to_show
                .into_iter()
                .filter(|repo| {
                    !selected_repos
                        .iter()
                        .any(|selected| selected.name == repo.name)
                })
                .collect();

            if available_repos.is_empty() {
                println!("All matching repositories are already selected.");
                continue;
            }

            // Display numbered list
            println!("Found {} repositories:", available_repos.len());
            for (i, repo) in available_repos.iter().enumerate() {
                println!(
                    "  {}. {} ({})",
                    i + 1,
                    repo.name,
                    Self::format_source(&repo.source)
                );
            }
            println!();

            // Get selection
            print!("Select repositories (numbers, ranges, 'all', or Enter to search again): ");
            io::stdout().flush()?;

            let mut selection_input = String::new();
            io::stdin().read_line(&mut selection_input)?;
            let selection = selection_input.trim();

            if selection.is_empty() {
                continue; // Search again
            }

            // Parse selection and add to selected repositories
            match Self::parse_selection(selection, &available_repos) {
                Ok(new_selections) => {
                    if !new_selections.is_empty() {
                        let names: Vec<String> =
                            new_selections.iter().map(|r| r.name.clone()).collect();
                        println!("✓ Added: {}", names.join(", "));
                        selected_repos.extend(new_selections);
                    }
                }
                Err(e) => {
                    println!("Invalid selection: {e}");
                    println!(
                        "Use: single numbers (3), comma-separated (1,3,5), ranges (1-5), or 'all'"
                    );
                }
            }
            println!();
        }

        Ok(selected_repos)
    }

    /// Show overview of available repositories grouped by source
    fn show_repository_overview(repositories: &[Repository]) {
        let groups = RepositorySearch::group_by_source(repositories);

        println!("Available repositories by source:");
        for (source, repos) in &groups {
            println!("  📂 {}: {} repositories", source, repos.len());
        }
    }

    /// Format repository source for display
    fn format_source(source: &str) -> String {
        if source.contains("GitHub") && source.contains('(') && source.contains(')') {
            // Extract account name from "GitHub (account)"
            if let Some(start) = source.find('(') {
                if let Some(end) = source.find(')') {
                    let account = &source[start + 1..end];
                    return format!("GitHub/{account}");
                }
            }
        }
        source.to_string()
    }

    /// Parse user selection input into repository indices
    fn parse_selection(
        input: &str,
        available_repos: &[Repository],
    ) -> Result<Vec<Repository>, String> {
        let input = input.trim();

        if input == "all" {
            return Ok(available_repos.to_vec());
        }

        let mut selected = Vec::new();
        let max_index = available_repos.len();

        // Split by comma or space
        let parts: Vec<&str> = input
            .split(&[',', ' '][..])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        for part in parts {
            if part.contains('-') {
                // Handle range (e.g., "1-5")
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() != 2 {
                    return Err(format!("Invalid range format: '{part}'"));
                }

                let start: usize = range_parts[0]
                    .parse()
                    .map_err(|_| format!("Invalid number: '{}'", range_parts[0]))?;
                let end: usize = range_parts[1]
                    .parse()
                    .map_err(|_| format!("Invalid number: '{}'", range_parts[1]))?;

                if start == 0 || end == 0 {
                    return Err("Numbers must start from 1".to_string());
                }
                if start > max_index || end > max_index {
                    return Err(format!("Numbers must be between 1 and {max_index}"));
                }
                if start > end {
                    return Err(format!("Invalid range: {start} is greater than {end}"));
                }

                for i in start..=end {
                    let repo = available_repos[i - 1].clone();
                    if !selected.iter().any(|r: &Repository| r.name == repo.name) {
                        selected.push(repo);
                    }
                }
            } else {
                // Handle single number
                let index: usize = part
                    .parse()
                    .map_err(|_| format!("Invalid number: '{part}'"))?;

                if index == 0 {
                    return Err("Numbers must start from 1".to_string());
                }
                if index > max_index {
                    return Err(format!("Number must be between 1 and {max_index}"));
                }

                let repo = available_repos[index - 1].clone();
                if !selected.iter().any(|r: &Repository| r.name == repo.name) {
                    selected.push(repo);
                }
            }
        }

        Ok(selected)
    }

    /// Confirm repository selection
    pub fn confirm_selection(repositories: &[Repository]) -> Result<bool> {
        if repositories.is_empty() {
            return Ok(false);
        }

        println!("\nYou have selected {} repositories:", repositories.len());
        for (i, repo) in repositories.iter().enumerate() {
            println!(
                "  {}. {} ({})",
                i + 1,
                repo.name,
                Self::format_source(&repo.source)
            );
        }

        loop {
            print!("Proceed with these repositories? (y/n) [y]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let response = input.trim().to_lowercase();

            match response.as_str() {
                "" | "y" | "yes" => return Ok(true),
                "n" | "no" => return Ok(false),
                _ => println!("Please enter 'y' for yes or 'n' for no"),
            }
        }
    }
}

impl Default for InteractiveSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repos() -> Vec<Repository> {
        vec![
            Repository {
                name: "repo1".to_string(),
                url: "git@github.com:user/repo1.git".to_string(),
                is_private: false,
                source: "GitHub (user)".to_string(),
                account: None,
            },
            Repository {
                name: "repo2".to_string(),
                url: "git@github.com:user/repo2.git".to_string(),
                is_private: false,
                source: "GitHub (user)".to_string(),
                account: None,
            },
            Repository {
                name: "repo3".to_string(),
                url: "git@github.com:user/repo3.git".to_string(),
                is_private: false,
                source: "GitHub (user)".to_string(),
                account: None,
            },
        ]
    }

    #[test]
    fn test_parse_selection_single_number() {
        let repos = create_test_repos();

        let result = InteractiveSelector::parse_selection("2", &repos).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "repo2");
    }

    #[test]
    fn test_parse_selection_comma_separated() {
        let repos = create_test_repos();

        let result = InteractiveSelector::parse_selection("1,3", &repos).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "repo1");
        assert_eq!(result[1].name, "repo3");
    }

    #[test]
    fn test_parse_selection_space_separated() {
        let repos = create_test_repos();

        let result = InteractiveSelector::parse_selection("1 3", &repos).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "repo1");
        assert_eq!(result[1].name, "repo3");
    }

    #[test]
    fn test_parse_selection_range() {
        let repos = create_test_repos();

        let result = InteractiveSelector::parse_selection("1-3", &repos).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "repo1");
        assert_eq!(result[1].name, "repo2");
        assert_eq!(result[2].name, "repo3");
    }

    #[test]
    fn test_parse_selection_all() {
        let repos = create_test_repos();

        let result = InteractiveSelector::parse_selection("all", &repos).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_selection_invalid_number() {
        let repos = create_test_repos();

        let result = InteractiveSelector::parse_selection("0", &repos);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must start from 1"));
    }

    #[test]
    fn test_parse_selection_out_of_range() {
        let repos = create_test_repos();

        let result = InteractiveSelector::parse_selection("5", &repos);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be between 1 and 3"));
    }

    #[test]
    fn test_format_source() {
        assert_eq!(
            InteractiveSelector::format_source("GitHub (dheater)"),
            "GitHub/dheater"
        );
        assert_eq!(
            InteractiveSelector::format_source("GitHub (work-account)"),
            "GitHub/work-account"
        );
        assert_eq!(InteractiveSelector::format_source("Local"), "Local");
    }
}
