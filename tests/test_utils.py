"""Test utilities and fixtures to reduce duplication and brittleness."""

import tempfile
from pathlib import Path
from unittest.mock import MagicMock, patch
from typing import Dict, List, Any
import pytest


class MockSubprocessResult:
    """Helper to create consistent subprocess results"""
    
    def __init__(self, returncode: int = 0, stdout: str = "", stderr: str = ""):
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


class GitConfigBuilder:
    """Builder pattern for creating git configurations in tests"""
    
    def __init__(self, temp_dir: Path):
        self.temp_dir = Path(temp_dir)
        self.configs = {}
    
    def add_context(self, name: str, user_name: str, email: str) -> 'GitConfigBuilder':
        """Add a git context configuration"""
        config_file = self.temp_dir / f".gitconfig-{name}"
        config_content = f'''[user]
    name = "{user_name}"
    email = "{email}"
'''
        config_file.write_text(config_content)
        self.configs[name] = {"name": user_name, "email": email}
        return self
    
    def build_main_config(self) -> Path:
        """Create the main .gitconfig with includeIf directives"""
        main_config = self.temp_dir / ".gitconfig"
        content = '[user]\n    useConfigOnly = true\n\n'
        
        for context_name in self.configs.keys():
            content += f'[includeIf "gitdir:~/src/{context_name}/"]\n'
            content += f'    path = ~/.gitconfig-{context_name}\n\n'
        
        main_config.write_text(content)
        return main_config


class ViewsetConfigBuilder:
    """Builder pattern for creating viewset configurations in tests"""
    
    def __init__(self, temp_dir: Path):
        self.temp_dir = Path(temp_dir)
        self.viewsets = {}
    
    def add_viewset(self, name: str, repos: List[Dict[str, str]]) -> 'ViewsetConfigBuilder':
        """Add a viewset with repositories"""
        self.viewsets[name] = {"repos": repos}
        return self
    
    def build_config(self) -> Path:
        """Create the viewsets.yaml file"""
        config_dir = self.temp_dir / ".config" / "viewyard"
        config_dir.mkdir(parents=True, exist_ok=True)
        config_file = config_dir / "viewsets.yaml"
        
        import yaml
        with open(config_file, 'w') as f:
            yaml.dump({"viewsets": self.viewsets}, f, default_flow_style=False)
        
        return config_file


class GitHubMocker:
    """Centralized GitHub CLI mocking to reduce duplication"""
    
    @staticmethod
    def create_auth_status_mock(accounts: List[str]) -> MockSubprocessResult:
        """Create mock for 'gh auth status' command"""
        stdout_lines = []
        for i, account in enumerate(accounts):
            active = "true" if i == 0 else "false"
            stdout_lines.append(f"  ✓ Logged in to github.com account {account} (keyring)")
            stdout_lines.append(f"  - Active account: {active}")
        
        return MockSubprocessResult(0, "\n".join(stdout_lines))
    
    @staticmethod
    def create_repo_list_mock(repos: List[Dict[str, Any]]) -> MockSubprocessResult:
        """Create mock for 'gh repo list' command"""
        import json
        return MockSubprocessResult(0, json.dumps(repos))
    
    @staticmethod
    def create_comprehensive_mock(accounts: List[str], repos_by_account: Dict[str, List[Dict]]):
        """Create a comprehensive GitHub CLI mock"""
        
        def side_effect(cmd, **kwargs):
            if cmd == ["gh", "--version"]:
                return MockSubprocessResult(0, "gh version 2.0.0")
            
            elif cmd == ["gh", "auth", "status"]:
                return GitHubMocker.create_auth_status_mock(accounts)
            
            elif cmd == ["gh", "api", "user", "--jq", ".login"]:
                return MockSubprocessResult(0, accounts[0])  # Current user
            
            elif cmd[:3] == ["gh", "auth", "switch"]:
                return MockSubprocessResult(0)  # Successful switch
            
            elif cmd[:3] == ["gh", "repo", "list"] and "--json" in cmd:
                # Return repos for the first account (simplified for testing)
                if repos_by_account:
                    first_account_repos = list(repos_by_account.values())[0]
                    return GitHubMocker.create_repo_list_mock(first_account_repos)
                return GitHubMocker.create_repo_list_mock([])
            
            elif cmd == ["gh", "api", "user/orgs", "--jq", ".[].login"]:
                return MockSubprocessResult(0, "")  # No orgs for simplicity
            
            else:
                return MockSubprocessResult(1, "", "Command not found")
        
        return side_effect


@pytest.fixture
def temp_home():
    """Fixture providing a temporary home directory"""
    with tempfile.TemporaryDirectory() as temp_dir:
        with patch('pathlib.Path.home', return_value=Path(temp_dir)):
            yield Path(temp_dir)


@pytest.fixture
def git_config_builder(temp_home):
    """Fixture providing a GitConfigBuilder"""
    return GitConfigBuilder(temp_home)


@pytest.fixture
def viewset_config_builder(temp_home):
    """Fixture providing a ViewsetConfigBuilder"""
    return ViewsetConfigBuilder(temp_home)


@pytest.fixture
def github_mocker():
    """Fixture providing GitHubMocker utilities"""
    return GitHubMocker


# Common test data
SAMPLE_REPOS = {
    "personal": [
        {"name": "dotfiles", "url": "https://github.com/dheater/dotfiles", "isPrivate": False},
        {"name": "personal-project", "url": "https://github.com/dheater/personal-project", "isPrivate": False}
    ],
    "work": [
        {"name": "audit", "url": "https://github.com/imprivata-pas/audit", "isPrivate": True},
        {"name": "universal-connection-manager", "url": "https://github.com/daniel-heater-imprivata/ucm", "isPrivate": False}
    ]
}

SAMPLE_VIEWSETS = {
    "dheater": [
        {"name": "dotfiles", "url": "https://github.com/dheater/dotfiles"},
        {"name": "conan-center-index", "url": "https://github.com/dheater/conan-center-index"}
    ],
    "work": [
        {"name": "api-service", "url": "git@github.com:company/api-service.git", "build": "make", "test": "make test"},
        {"name": "web-app", "url": "git@github.com:company/web-app.git", "build": "npm run build", "test": "npm test"}
    ]
}
