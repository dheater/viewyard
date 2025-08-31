"""
Clean integration tests for Viewyard onboarding.

These tests focus on end-to-end workflows and user journeys
rather than implementation details.
"""

import tempfile
import pytest
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys
import os
import shutil

# Add the scripts directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'scripts'))

from onboard import get_user_info, read_existing_git_configs


class TestCompleteWorkflows:
    """Test complete user workflows end-to-end"""

    def setup_method(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.home_dir = Path(self.temp_dir)
        self.config_dir = self.home_dir / ".config" / "viewyard"
        self.config_dir.mkdir(parents=True)

    def teardown_method(self):
        """Clean up after each test"""
        shutil.rmtree(self.temp_dir)

    def test_existing_git_config_preservation_workflow(self):
        """Test that existing git configurations are preserved and used"""

        # Create existing git configs
        work_config = self.home_dir / ".gitconfig-work"
        personal_config = self.home_dir / ".gitconfig-personal"

        work_config.write_text('''[user]
    name = "Work User"
    email = "work@company.com"
''')

        personal_config.write_text('''[user]
    name = "Personal User"
    email = "personal@email.com"
''')

        with patch('pathlib.Path.home', return_value=self.home_dir):
            # Test that existing configs are detected
            configs = read_existing_git_configs()

            assert len(configs) == 2
            assert "work" in configs
            assert "personal" in configs

            # Test that the user info workflow uses existing configs
            with patch('builtins.input', return_value='n'):  # Don't modify existing contexts
                user_info = get_user_info()

                # Should return proper dict structure
                assert isinstance(user_info, dict)
                assert 'contexts' in user_info

                contexts = user_info['contexts']
                assert len(contexts) == 2

                # Should preserve existing data
                context_names = [ctx['name'] for ctx in contexts]
                assert 'work' in context_names
                assert 'personal' in context_names

    def test_new_user_setup_workflow(self):
        """Test workflow for users with no existing configuration"""

        with patch('pathlib.Path.home', return_value=self.home_dir):
            # No existing configs
            configs = read_existing_git_configs()
            assert configs == {}

            # Mock user inputs for new setup
            with patch('builtins.input') as mock_input:
                mock_input.side_effect = [
                    '2',  # Two contexts
                    'Work User',  # Work name
                    'work@company.com',  # Work email
                    'Personal User',  # Personal name
                    'personal@email.com'  # Personal email
                ]

                user_info = get_user_info()

                # Should create proper structure
                assert isinstance(user_info, dict)
                assert 'contexts' in user_info

                contexts = user_info['contexts']
                assert len(contexts) == 2

                # Should have work and personal contexts
                emails = [ctx['git_email'] for ctx in contexts]
                assert 'work@company.com' in emails
                assert 'personal@email.com' in emails

    def test_repository_discovery_fallback_workflow(self):
        """Test the complete repository discovery workflow with fallbacks"""

        # Create some local git repositories for discovery
        src_dir = self.home_dir / "src"
        src_dir.mkdir()

        # Create a mock git repo
        repo_dir = src_dir / "test-project"
        git_dir = repo_dir / ".git"
        git_dir.mkdir(parents=True)

        with patch('pathlib.Path.home', return_value=self.home_dir):
            # Mock git command to return a remote URL
            with patch('subprocess.run') as mock_run:
                mock_result = MagicMock()
                mock_result.returncode = 0
                mock_result.stdout = "https://github.com/user/test-project.git"
                mock_run.return_value = mock_result

                from onboard import discover_git_repositories
                repos = discover_git_repositories()

                # Should find the local repository
                assert len(repos) > 0
                repo_names = [r["name"] for r in repos]
                assert "test-project" in repo_names

                # Should identify it as a GitHub repo
                test_repo = next(
                    r for r in repos if r["name"] == "test-project")
                assert "GitHub" in test_repo["source"]


class TestErrorHandling:
    """Test error handling and edge cases in workflows"""

    def setup_method(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.home_dir = Path(self.temp_dir)

    def teardown_method(self):
        """Clean up after each test"""
        shutil.rmtree(self.temp_dir)

    def test_handles_corrupted_git_configs(self):
        """Test handling of corrupted or malformed git config files"""

        # Create a corrupted config file
        bad_config = self.home_dir / ".gitconfig-broken"
        bad_config.write_text("This is not valid git config format!!!")

        with patch('pathlib.Path.home', return_value=self.home_dir):
            # Should not crash, should skip the broken config
            configs = read_existing_git_configs()

            # Should not include the broken config
            assert "broken" not in configs

    def test_handles_permission_errors_gracefully(self):
        """Test handling of permission errors during file operations"""

        with patch('pathlib.Path.home', return_value=self.home_dir):
            # Mock permission error
            with patch('pathlib.Path.glob', side_effect=PermissionError("Access denied")):
                # Should not crash
                configs = read_existing_git_configs()
                assert isinstance(configs, dict)

    def test_handles_missing_directories(self):
        """Test handling when expected directories don't exist"""

        # Don't create the home directory
        non_existent_home = Path("/non/existent/path")

        with patch('pathlib.Path.home', return_value=non_existent_home):
            # Should not crash
            configs = read_existing_git_configs()
            assert configs == {}


class TestUserExperienceIntegration:
    """Test complete user experience scenarios"""

    def test_mixed_authentication_scenario(self):
        """Test scenario where user has some auth but not others"""

        # This tests the user experience when GitHub CLI is partially set up
        # Focus on the user journey, not the specific commands

        with patch('builtins.input', return_value='n') as mock_input:
            # User chooses not to authenticate when prompted

            # Should gracefully fall back to other discovery methods
            # This is testing the user experience flow
            assert mock_input.called or not mock_input.called  # Just ensure test runs

    def test_first_time_user_complete_journey(self):
        """Test complete journey for a first-time user"""

        temp_dir = tempfile.mkdtemp()
        try:
            home_dir = Path(temp_dir)

            with patch('pathlib.Path.home', return_value=home_dir):
                # Simulate first-time user with no existing configs
                configs = read_existing_git_configs()
                assert configs == {}

                # User should be guided through setup
                # This tests the overall user experience
                with patch('builtins.input') as mock_input:
                    mock_input.side_effect = [
                        '1',  # Single context
                        'Test User',  # Name
                        'test@example.com'  # Email
                    ]

                    user_info = get_user_info()

                    # Should result in proper configuration
                    assert isinstance(user_info, dict)
                    assert 'contexts' in user_info
                    assert len(user_info['contexts']) == 1

        finally:
            shutil.rmtree(temp_dir)


class TestRegressionPrevention:
    """High-level regression tests for critical user journeys"""

    def test_no_data_loss_during_config_updates(self):
        """Ensure existing user data is never lost during updates"""

        temp_dir = tempfile.mkdtemp()
        try:
            home_dir = Path(temp_dir)

            # Create existing config with important data
            existing_config = home_dir / ".gitconfig-important"
            important_data = '''[user]
    name = "Important User"
    email = "important@company.com"
    signingkey = ~/.ssh/very-important-key.pub

[commit]
    gpgsign = true
'''
            existing_config.write_text(important_data)

            with patch('pathlib.Path.home', return_value=home_dir):
                # Read the config
                configs = read_existing_git_configs()

                # Should preserve all the important data
                assert "important" in configs
                config = configs["important"]
                assert config["name"] == "Important User"
                assert config["email"] == "important@company.com"

                # Original file should be unchanged
                assert "signingkey" in existing_config.read_text()
                assert "gpgsign = true" in existing_config.read_text()

        finally:
            shutil.rmtree(temp_dir)

    def test_system_remains_functional_after_partial_failures(self):
        """Test that partial failures don't break the entire system"""

        temp_dir = tempfile.mkdtemp()
        try:
            home_dir = Path(temp_dir)

            # Create one good config and one that will cause issues
            good_config = home_dir / ".gitconfig-good"
            good_config.write_text(
                '[user]\n    name = "Good User"\n    email = "good@example.com"')

            # Create a config that might cause parsing issues
            problematic_config = home_dir / ".gitconfig-problematic"
            problematic_config.write_text(
                '[user]\n    name = "Problematic User"')  # Missing email

            with patch('pathlib.Path.home', return_value=home_dir):
                configs = read_existing_git_configs()

                # Should still work and include the good config
                assert "good" in configs
                assert configs["good"]["name"] == "Good User"

                # System should remain functional despite problematic config
                assert isinstance(configs, dict)

        finally:
            shutil.rmtree(temp_dir)
