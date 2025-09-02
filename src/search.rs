use crate::models::Repository;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct RepositorySearch {
    matcher: SkimMatcherV2,
}

impl RepositorySearch {
    #[must_use]
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search repositories with fuzzy matching
    pub fn search(&self, repositories: &[Repository], query: &str) -> Vec<(Repository, i64)> {
        if query.trim().is_empty() {
            return repositories.iter().map(|repo| (repo.clone(), 0)).collect();
        }

        let mut matches = Vec::new();

        for repo in repositories {
            if let Some(score) = self.matcher.fuzzy_match(&repo.name, query) {
                matches.push((repo.clone(), score));
            }
        }

        // Sort by score (descending - higher scores are better matches)
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        matches
    }

    /// Group repositories by source for better display
    pub fn group_by_source(
        repositories: &[Repository],
    ) -> std::collections::BTreeMap<String, Vec<Repository>> {
        let mut groups = std::collections::BTreeMap::new();

        for repo in repositories {
            let source_key = if repo.source.contains("GitHub (") {
                // Extract account/org from "GitHub (account)" or "GitHub (org/account)"
                repo.source.find("GitHub (").map_or_else(
                    || repo.source.clone(),
                    |start| {
                        let after_github = &repo.source[start + 8..];
                        after_github.find(')').map_or_else(
                            || repo.source.clone(),
                            |end| {
                                let account_part = &after_github[..end];
                                if account_part.contains('/') {
                                    // Organization repo: "org/account"
                                    let org =
                                        account_part.split('/').next().unwrap_or(account_part);
                                    format!("GitHub ({org})")
                                } else {
                                    // Personal repo: "account"
                                    format!("GitHub ({account_part})")
                                }
                            },
                        )
                    },
                )
            } else {
                repo.source.clone()
            };

            groups
                .entry(source_key)
                .or_insert_with(Vec::new)
                .push(repo.clone());
        }

        groups
    }
}

impl Default for RepositorySearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repo(name: &str, source: &str) -> Repository {
        Repository {
            name: name.to_string(),
            url: format!("https://github.com/test/{name}"),
            is_private: false,
            source: source.to_string(),
        }
    }

    #[test]
    fn test_fuzzy_search() {
        let search = RepositorySearch::new();
        let repos = vec![
            create_test_repo("my-awesome-project", "GitHub (dheater)"),
            create_test_repo("another-project", "GitHub (dheater)"),
            create_test_repo("work-project", "GitHub (imprivata/dheater)"),
        ];

        let results = search.search(&repos, "awesome");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.name, "my-awesome-project");
    }

    #[test]
    fn test_grouping_by_source() {
        let repos = vec![
            create_test_repo("repo1", "GitHub (dheater)"),
            create_test_repo("repo2", "GitHub (dheater)"),
            create_test_repo("repo3", "GitHub (imprivata/dheater)"),
        ];

        let groups = RepositorySearch::group_by_source(&repos);
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("GitHub (dheater)"));
        assert!(groups.contains_key("GitHub (imprivata)"));
    }
}
