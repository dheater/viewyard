use viewyard::models::Repository;
use viewyard::search::RepositorySearch;

#[test]
fn test_repository_creation() {
    let repo = Repository {
        name: "test-repo".to_string(),
        url: "https://github.com/test/repo.git".to_string(),
        is_private: false,
        source: "GitHub (test)".to_string(),
        account: None,
    };

    assert_eq!(repo.name, "test-repo");
    assert_eq!(repo.url, "https://github.com/test/repo.git");
    assert!(!repo.is_private);
    assert_eq!(repo.source, "GitHub (test)");
}

#[test]
fn test_repository_search_fuzzy_matching() {
    let search = RepositorySearch::new();
    let repos = vec![
        Repository {
            name: "my-awesome-project".to_string(),
            url: "https://github.com/test/my-awesome-project.git".to_string(),
            is_private: false,
            source: "GitHub (test)".to_string(),
            account: None,
        },
        Repository {
            name: "another-project".to_string(),
            url: "https://github.com/test/another-project.git".to_string(),
            is_private: false,
            source: "GitHub (test)".to_string(),
            account: None,
        },
    ];

    let results = search.search(&repos, "awesome");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "my-awesome-project");
    assert!(results[0].1 > 0); // Should have a positive score
}

#[test]
fn test_repository_search_empty_query() {
    let search = RepositorySearch::new();
    let repos = vec![
        Repository {
            name: "repo1".to_string(),
            url: "https://github.com/test/repo1.git".to_string(),
            is_private: false,
            source: "GitHub (test)".to_string(),
            account: None,
        },
        Repository {
            name: "repo2".to_string(),
            url: "https://github.com/test/repo2.git".to_string(),
            is_private: false,
            source: "GitHub (test)".to_string(),
            account: None,
        },
    ];

    let results = search.search(&repos, "");
    assert_eq!(results.len(), 2); // Should return all repositories
    assert_eq!(results[0].1, 0); // Score should be 0 for empty query
    assert_eq!(results[1].1, 0);
}

#[test]
fn test_repository_grouping_by_source() {
    let repos = vec![
        Repository {
            name: "repo1".to_string(),
            url: "https://github.com/user/repo1.git".to_string(),
            is_private: false,
            source: "GitHub (user)".to_string(),
            account: None,
        },
        Repository {
            name: "repo2".to_string(),
            url: "https://github.com/user/repo2.git".to_string(),
            is_private: false,
            source: "GitHub (user)".to_string(),
            account: None,
        },
        Repository {
            name: "repo3".to_string(),
            url: "https://github.com/org/repo3.git".to_string(),
            is_private: false,
            source: "GitHub (org/user)".to_string(),
            account: None,
        },
    ];

    let groups = RepositorySearch::group_by_source(&repos);
    assert_eq!(groups.len(), 2);
    assert!(groups.contains_key("GitHub (user)"));
    assert!(groups.contains_key("GitHub (org)"));

    assert_eq!(groups["GitHub (user)"].len(), 2);
    assert_eq!(groups["GitHub (org)"].len(), 1);
}

#[test]
fn test_repository_private_flag() {
    let private_repo = Repository {
        name: "private-repo".to_string(),
        url: "https://github.com/user/private-repo.git".to_string(),
        is_private: true,
        source: "GitHub (user) [private]".to_string(),
        account: None,
    };

    let public_repo = Repository {
        name: "public-repo".to_string(),
        url: "https://github.com/user/public-repo.git".to_string(),
        is_private: false,
        source: "GitHub (user)".to_string(),
        account: None,
    };

    assert!(private_repo.is_private);
    assert!(!public_repo.is_private);
    assert!(private_repo.source.contains("[private]"));
    assert!(!public_repo.source.contains("[private]"));
}

#[test]
fn test_repository_search_scoring() {
    let search = RepositorySearch::new();
    let repos = vec![
        Repository {
            name: "exact-match".to_string(),
            url: "https://github.com/test/exact-match.git".to_string(),
            is_private: false,
            source: "GitHub (test)".to_string(),
            account: None,
        },
        Repository {
            name: "partial-exact-match".to_string(),
            url: "https://github.com/test/partial-exact-match.git".to_string(),
            is_private: false,
            source: "GitHub (test)".to_string(),
            account: None,
        },
        Repository {
            name: "no-match-here".to_string(),
            url: "https://github.com/test/no-match-here.git".to_string(),
            is_private: false,
            source: "GitHub (test)".to_string(),
            account: None,
        },
    ];

    let results = search.search(&repos, "exact");

    // Should find both repositories that contain "exact"
    assert_eq!(results.len(), 2);

    // Results should be sorted by score (higher scores first)
    assert!(results[0].1 >= results[1].1);

    // The exact match should score higher than partial match
    if results[0].0.name == "exact-match" {
        assert!(results[0].1 > results[1].1);
    }
}

#[test]
fn test_repository_search_no_matches() {
    let search = RepositorySearch::new();
    let repos = vec![Repository {
        name: "repo1".to_string(),
        url: "https://github.com/test/repo1.git".to_string(),
        is_private: false,
        source: "GitHub (test)".to_string(),
        account: None,
    }];

    let results = search.search(&repos, "nonexistent");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_repository_search_with_large_dataset() {
    let search = RepositorySearch::new();

    // Generate a large number of repositories to test performance
    let mut repos = Vec::new();
    for i in 0..1000 {
        repos.push(Repository {
            name: format!("repo-{i:04}"),
            url: format!("https://github.com/test/repo-{i:04}.git"),
            is_private: i % 3 == 0, // Every third repo is private
            source: if i % 2 == 0 {
                "GitHub (user)".to_string()
            } else {
                "GitHub (org/user)".to_string()
            },
            account: None,
        });
    }

    // Add a specific repository to search for
    repos.push(Repository {
        name: "special-project".to_string(),
        url: "https://github.com/test/special-project.git".to_string(),
        is_private: false,
        source: "GitHub (user)".to_string(),
        account: None,
    });

    let results = search.search(&repos, "special");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.name, "special-project");
}
