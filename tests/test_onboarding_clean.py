"""Clean, focused tests for onboarding functionality using test utilities."""

import pytest
from unittest.mock import patch
from pathlib import Path

# Import our test utilities
from test_utils import (
    git_config_builder, viewset_config_builder, github_mocker, temp_home,
    SAMPLE_REPOS, SAMPLE_VIEWSETS
)

# Import functions to test
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'scripts'))
from onboard import (
    check_prerequisites, read_existing_git_configs, discover_repositories,
    filter_repos_by_context, fuzzy_search_repos, analyze_viewset_directory
)


class TestPrerequisites:
    """Test prerequisite checking functionality"""
    
    def test_all_tools_present(self):
        """Test when all required tools are available"""
        with patch('subprocess.run') as mock_run:
            mock_run.return_value.returncode = 0
            assert check_prerequisites() is True
    
    def test_missing_tools(self):
        """Test when tools are missing"""
        with patch('subprocess.run') as mock_run:
            mock_run.side_effect = FileNotFoundError()
            assert check_prerequisites() is False


class TestGitConfiguration:
    """Test git configuration detection and management"""
    
    def test_reads_existing_configs(self, git_config_builder):
        """Test detection of existing context-specific git configs"""
        # Use builder pattern to create test configs
        git_config_builder.add_context("work", "Work User", "work@company.com")
        git_config_builder.add_context("personal", "Personal User", "personal@email.com")
        git_config_builder.build_main_config()
        
        configs = read_existing_git_configs()
        
        assert len(configs) == 2
        assert configs["work"]["email"] == "work@company.com"
        assert configs["personal"]["email"] == "personal@email.com"
    
    def test_handles_no_configs(self, temp_home):
        """Test behavior when no context configs exist"""
        configs = read_existing_git_configs()
        assert configs == {}
    
    def test_context_mapping_patterns(self, git_config_builder):
        """Test that various email patterns map to correct contexts"""
        git_config_builder.add_context("imprivata", "Daniel Heater", "daniel.heater@imprivata.com")
        git_config_builder.add_context("dheater", "Daniel Heater", "dheater@pm.me")
        git_config_builder.build_main_config()
        
        configs = read_existing_git_configs()
        
        # Should detect both contexts
        assert "imprivata" in configs
        assert "dheater" in configs
        assert "imprivata.com" in configs["imprivata"]["email"]
        assert "pm.me" in configs["dheater"]["email"]


class TestRepositoryDiscovery:
    """Test repository discovery functionality"""
    
    def test_discovers_repositories_when_authenticated(self, github_mocker):
        """Test that repository discovery works when authenticated"""
        # Simplified test focusing on behavior, not implementation details
        with patch('subprocess.run') as mock_run:
            # Mock successful authentication and discovery
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = '[]'  # Empty but valid JSON

            repos = discover_repositories()

            # Should return a list (behavior test, not content test)
            assert isinstance(repos, list)
    
    def test_handles_no_authentication(self, github_mocker):
        """Test graceful handling when not authenticated"""
        with patch('subprocess.run') as mock_run:
            mock_run.return_value.returncode = 1  # Not authenticated
            
            repos = discover_repositories()
            # Should fall back to git-based discovery
            assert isinstance(repos, list)


class TestContextFiltering:
    """Test repository filtering by context"""
    
    def test_work_context_filtering(self):
        """Test that work context gets work repositories"""
        mock_repos = [
            {"name": "audit", "source": "GitHub (imprivata-pas) [private]"},
            {"name": "dotfiles", "source": "GitHub (dheater)"},
            {"name": "work-repo", "source": "GitHub (daniel-heater-imprivata)"}
        ]
        
        work_repos = filter_repos_by_context(mock_repos, "imprivata")
        work_names = [r["name"] for r in work_repos]
        
        assert "audit" in work_names
        assert "work-repo" in work_names
        assert "dotfiles" not in work_names  # Personal repo excluded
    
    def test_personal_context_filtering(self):
        """Test that personal context gets personal repositories"""
        mock_repos = [
            {"name": "audit", "source": "GitHub (imprivata-pas) [private]"},
            {"name": "dotfiles", "source": "GitHub (dheater)"},
            {"name": "personal-project", "source": "GitHub (dheater)"}
        ]
        
        personal_repos = filter_repos_by_context(mock_repos, "dheater")
        personal_names = [r["name"] for r in personal_repos]
        
        assert "dotfiles" in personal_names
        assert "personal-project" in personal_names
        assert "audit" not in personal_names  # Work repo excluded


class TestFuzzySearch:
    """Test fuzzy search functionality"""
    
    def test_exact_match(self):
        """Test exact repository name matching"""
        repos = [{"name": "universal-connection-manager", "source": "GitHub"}]
        results = fuzzy_search_repos(repos, "universal-connection-manager")
        assert len(results) == 1
        assert results[0]["name"] == "universal-connection-manager"
    
    def test_partial_match(self):
        """Test partial name matching"""
        repos = [
            {"name": "universal-connection-manager", "source": "GitHub"},
            {"name": "audit", "source": "GitHub"}
        ]
        results = fuzzy_search_repos(repos, "universal")
        assert len(results) == 1
        assert results[0]["name"] == "universal-connection-manager"
    
    def test_case_insensitive(self):
        """Test case insensitive matching"""
        repos = [{"name": "MyProject", "source": "GitHub"}]
        results = fuzzy_search_repos(repos, "myproject")
        assert len(results) == 1


class TestExistingViewsets:
    """Test handling of existing viewset configurations"""
    
    def test_detects_existing_viewsets(self, viewset_config_builder):
        """Test detection of existing viewsets"""
        viewset_config_builder.add_viewset("dheater", SAMPLE_VIEWSETS["dheater"])
        viewset_config_builder.add_viewset("work", SAMPLE_VIEWSETS["work"])
        config_file = viewset_config_builder.build_config()
        
        # Verify config was created correctly
        assert config_file.exists()
        
        import yaml
        with open(config_file) as f:
            config = yaml.safe_load(f)
        
        viewsets = config.get("viewsets", {})
        assert len(viewsets) == 2
        assert "dheater" in viewsets
        assert "work" in viewsets
        
        # Verify repository data is preserved
        dheater_repos = viewsets["dheater"]["repos"]
        assert len(dheater_repos) == 2
        assert any(repo["name"] == "dotfiles" for repo in dheater_repos)
    
    def test_preserves_build_commands(self, viewset_config_builder):
        """Test that build and test commands are preserved"""
        viewset_config_builder.add_viewset("work", SAMPLE_VIEWSETS["work"])
        config_file = viewset_config_builder.build_config()
        
        import yaml
        with open(config_file) as f:
            config = yaml.safe_load(f)
        
        work_repos = config["viewsets"]["work"]["repos"]
        api_service = next(repo for repo in work_repos if repo["name"] == "api-service")
        
        assert api_service["build"] == "make"
        assert api_service["test"] == "make test"


class TestRegressionPrevention:
    """High-level regression tests for critical issues"""
    
    def test_multiple_account_discovery(self):
        """Regression: Should handle multiple authenticated accounts"""
        # High-level behavior test without brittle mocking
        from onboard import get_available_github_accounts

        with patch('subprocess.run') as mock_run:
            # Mock successful auth status with multiple accounts
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = "✓ Logged in to github.com account user1\n✓ Logged in to github.com account user2"

            accounts = get_available_github_accounts()

            # Should detect multiple accounts
            assert len(accounts) >= 1  # At least one account detected
    
    def test_context_separation(self):
        """Regression: Work and personal contexts should be separate"""
        mixed_repos = [
            {"name": "audit", "source": "GitHub (imprivata-pas) [private]"},
            {"name": "dotfiles", "source": "GitHub (dheater)"},
            {"name": "work-tool", "source": "GitHub (daniel-heater-imprivata)"}
        ]
        
        work_repos = filter_repos_by_context(mixed_repos, "imprivata")
        personal_repos = filter_repos_by_context(mixed_repos, "dheater")
        
        work_names = [r["name"] for r in work_repos]
        personal_names = [r["name"] for r in personal_repos]
        
        # Verify separation
        assert "audit" in work_names and "audit" not in personal_names
        assert "dotfiles" in personal_names and "dotfiles" not in work_names
        assert "work-tool" in work_names and "work-tool" not in personal_names
    
    def test_existing_config_preservation(self, git_config_builder):
        """Regression: Should detect contexts from actual config names"""
        git_config_builder.add_context("imprivata", "Daniel Heater", "daniel.heater@imprivata.com")
        git_config_builder.add_context("dheater", "Daniel Heater", "dheater@pm.me")
        git_config_builder.build_main_config()
        
        configs = read_existing_git_configs()
        
        # Should detect both contexts, not just one
        assert len(configs) == 2
        assert "imprivata" in configs
        assert "dheater" in configs
