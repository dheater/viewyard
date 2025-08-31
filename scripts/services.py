"""Service layer for Viewyard onboarding business logic."""

import subprocess
import json
import yaml
from pathlib import Path
from typing import List, Dict, Optional, Any

from models import (
    Repository, GitContext, Viewset, OnboardingState, RepositorySearchResult,
    DirectoryAnalysis, PrerequisiteError, GitConfigError, RepositoryDiscoveryError,
    ViewsetConfigError
)


class PrerequisiteService:
    """Service for checking and managing prerequisites"""
    
    REQUIRED_TOOLS = ["git", "just", "python3"]
    OPTIONAL_TOOLS = ["gh"]
    
    @staticmethod
    def check_all() -> bool:
        """Check if all required tools are available"""
        try:
            for tool in PrerequisiteService.REQUIRED_TOOLS:
                result = subprocess.run([tool, "--version"], 
                                      capture_output=True, timeout=5)
                if result.returncode != 0:
                    return False
            return True
        except (FileNotFoundError, subprocess.TimeoutExpired):
            return False
    
    @staticmethod
    def check_python_packages() -> bool:
        """Check if required Python packages are available"""
        try:
            import yaml
            return True
        except ImportError:
            return False


class GitService:
    """Service for git configuration management"""
    
    @staticmethod
    def read_existing_contexts() -> List[GitContext]:
        """Read existing git contexts from config files"""
        contexts = []
        home = Path.home()
        
        # Look for .gitconfig-* files
        for config_file in home.glob(".gitconfig-*"):
            context_name = config_file.name.replace(".gitconfig-", "")
            
            try:
                with open(config_file) as f:
                    content = f.read()
                
                # Simple parsing to extract name and email
                name_match = None
                email_match = None
                for line in content.split('\n'):
                    line = line.strip()
                    if line.startswith('name = '):
                        name_match = line.split('name = ')[1].strip('"')
                    elif line.startswith('email = '):
                        email_match = line.split('email = ')[1].strip('"')
                
                if name_match and email_match:
                    contexts.append(GitContext(
                        name=context_name,
                        user_name=name_match,
                        email=email_match,
                        directory=context_name  # Default to context name
                    ))
            except Exception:
                continue
        
        return contexts
    
    @staticmethod
    def create_context_config(context: GitContext) -> None:
        """Create git config file for a context"""
        config_content = f'''[user]
    name = "{context.user_name}"
    email = "{context.email}"
'''
        try:
            with open(context.config_file_path, 'w') as f:
                f.write(config_content)
        except Exception as e:
            raise GitConfigError(f"Failed to create git config for {context.name}: {e}")
    
    @staticmethod
    def update_main_config(contexts: List[GitContext]) -> None:
        """Update main .gitconfig with includeIf directives"""
        main_config = Path.home() / ".gitconfig"
        
        # Read existing config
        existing_content = ""
        if main_config.exists():
            with open(main_config) as f:
                existing_content = f.read()
        
        # Add includeIf sections for each context
        for context in contexts:
            include_section = f'''
[includeIf "gitdir:~/src/{context.directory}/"]
    path = ~/.gitconfig-{context.name}
'''
            if include_section.strip() not in existing_content:
                existing_content += include_section
        
        try:
            with open(main_config, 'w') as f:
                f.write(existing_content)
        except Exception as e:
            raise GitConfigError(f"Failed to update main git config: {e}")


class GitHubService:
    """Service for GitHub repository discovery"""
    
    @staticmethod
    def get_available_accounts() -> List[str]:
        """Get list of available GitHub accounts"""
        try:
            result = subprocess.run(["gh", "auth", "status"], 
                                  capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                accounts = []
                for line in result.stdout.split('\n'):
                    if "✓ Logged in to github.com account" in line:
                        parts = line.split("account ")
                        if len(parts) > 1:
                            account = parts[1].split(" ")[0].strip()
                            if account:
                                accounts.append(account)
                return accounts
        except Exception:
            pass
        return []
    
    @staticmethod
    def discover_repositories_from_account(account: str) -> List[Repository]:
        """Discover repositories from a specific GitHub account"""
        repos = []
        
        try:
            # Switch to the specified account
            switch_result = subprocess.run(["gh", "auth", "switch", "--user", account],
                                         capture_output=True, text=True, timeout=10)
            if switch_result.returncode != 0:
                return repos
            
            # Get user repositories
            user_result = subprocess.run([
                "gh", "repo", "list", "--limit", "100", "--json", "name,url,isPrivate"
            ], capture_output=True, text=True, timeout=30)
            
            if user_result.returncode == 0:
                user_repos = json.loads(user_result.stdout)
                for repo in user_repos:
                    privacy = " [private]" if repo.get("isPrivate", False) else ""
                    repos.append(Repository(
                        name=repo["name"],
                        url=repo["url"],
                        source=f"GitHub ({account}){privacy}"
                    ))
            
            # Get organization repositories
            orgs_result = subprocess.run([
                "gh", "api", "user/orgs", "--jq", ".[].login"
            ], capture_output=True, text=True, timeout=15)
            
            if orgs_result.returncode == 0:
                orgs = [org.strip() for org in orgs_result.stdout.split('\n') if org.strip()]
                
                for org in orgs:
                    org_result = subprocess.run([
                        "gh", "repo", "list", org, "--limit", "200", "--json", "name,url,isPrivate"
                    ], capture_output=True, text=True, timeout=30)
                    
                    if org_result.returncode == 0:
                        org_repos = json.loads(org_result.stdout)
                        for repo in org_repos:
                            privacy = " [private]" if repo.get("isPrivate", False) else ""
                            repos.append(Repository(
                                name=repo["name"],
                                url=repo["url"],
                                source=f"GitHub ({account}){privacy}"
                            ))
        
        except Exception:
            pass
        
        return repos
    
    @staticmethod
    def discover_all_repositories() -> List[Repository]:
        """Discover repositories from all available GitHub accounts"""
        all_repos = []
        accounts = GitHubService.get_available_accounts()
        
        if not accounts:
            return all_repos
        
        # Remember original account
        original_account = None
        try:
            current_result = subprocess.run(["gh", "api", "user", "--jq", ".login"],
                                          capture_output=True, text=True, timeout=5)
            if current_result.returncode == 0:
                original_account = current_result.stdout.strip()
        except:
            pass
        
        # Discover from each account
        for account in accounts:
            account_repos = GitHubService.discover_repositories_from_account(account)
            all_repos.extend(account_repos)
        
        # Restore original account
        if original_account and original_account in accounts:
            try:
                subprocess.run(["gh", "auth", "switch", "--user", original_account],
                             capture_output=True, text=True, timeout=5)
            except:
                pass
        
        return all_repos


class RepositoryService:
    """Service for repository management and search"""
    
    @staticmethod
    def filter_by_context(repositories: List[Repository], context: str) -> List[Repository]:
        """Filter repositories based on context (work vs personal)"""
        is_work_context = any(work_indicator in context.lower() for work_indicator in 
                             ['work', 'company', 'imprivata', 'daniel-heater-imprivata'])
        
        filtered_repos = []
        
        for repo in repositories:
            source = repo.source.lower()
            
            if is_work_context:
                # Work context: include organization repos and work GitHub accounts
                if (any(org in source for org in ["imprivata", "company"]) or
                    "daniel-heater-imprivata" in source or
                    "local" in source):
                    # But exclude personal account repos
                    if "github (dheater)" not in source:
                        filtered_repos.append(repo)
            else:
                # Personal context: include personal GitHub accounts and local repos
                if ("github (dheater)" in source or "local" in source):
                    # But exclude work organization repos and work accounts
                    if not any(org in source for org in ["imprivata", "company", "daniel-heater-imprivata"]):
                        filtered_repos.append(repo)
        
        return filtered_repos
    
    @staticmethod
    def fuzzy_search(repositories: List[Repository], query: str) -> RepositorySearchResult:
        """Perform fuzzy search on repositories"""
        if not query:
            return RepositorySearchResult(query="", results=[], total_available=len(repositories))
        
        query_lower = query.lower()
        results = []
        
        for repo in repositories:
            if query_lower in repo.name.lower():
                results.append(repo)
        
        return RepositorySearchResult(
            query=query,
            results=results,
            total_available=len(repositories)
        )


class ViewsetService:
    """Service for viewset configuration management"""

    @staticmethod
    def get_config_file_path() -> Path:
        """Get path to viewsets configuration file"""
        return Path.home() / ".config" / "viewyard" / "viewsets.yaml"

    @staticmethod
    def load_existing_viewsets() -> Dict[str, Viewset]:
        """Load existing viewsets from configuration file"""
        config_file = ViewsetService.get_config_file_path()
        viewsets = {}

        if config_file.exists():
            try:
                with open(config_file) as f:
                    config = yaml.safe_load(f)

                for name, data in config.get("viewsets", {}).items():
                    viewsets[name] = Viewset.from_dict(name, data)
            except Exception:
                pass

        return viewsets

    @staticmethod
    def save_viewsets(viewsets: Dict[str, Viewset]) -> None:
        """Save viewsets to configuration file"""
        config_file = ViewsetService.get_config_file_path()
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_data = {
            "viewsets": {name: viewset.to_dict() for name, viewset in viewsets.items()}
        }

        try:
            with open(config_file, 'w') as f:
                yaml.dump(config_data, f, default_flow_style=False)
        except Exception as e:
            raise ViewsetConfigError(f"Failed to save viewsets configuration: {e}")

    @staticmethod
    def analyze_directory(viewset_dir: Path) -> DirectoryAnalysis:
        """Analyze a viewset directory for existing content"""
        if not viewset_dir.exists():
            return DirectoryAnalysis(
                path=viewset_dir,
                exists=False,
                is_empty=True,
                has_git_repos=False,
                existing_repos=[],
                safe_to_proceed=True,
                message="Directory will be created"
            )

        # Check if directory is empty
        contents = list(viewset_dir.iterdir())
        is_empty = len(contents) == 0

        if is_empty:
            return DirectoryAnalysis(
                path=viewset_dir,
                exists=True,
                is_empty=True,
                has_git_repos=False,
                existing_repos=[],
                safe_to_proceed=True,
                message="Directory is empty and ready to use"
            )

        # Check for git repositories
        git_repos = []
        for item in contents:
            if item.is_dir() and (item / ".git").exists():
                git_repos.append(item.name)

        has_git_repos = len(git_repos) > 0

        if has_git_repos:
            return DirectoryAnalysis(
                path=viewset_dir,
                exists=True,
                is_empty=False,
                has_git_repos=True,
                existing_repos=git_repos,
                safe_to_proceed=False,
                message=f"Directory contains {len(git_repos)} git repositories that can be imported"
            )
        else:
            return DirectoryAnalysis(
                path=viewset_dir,
                exists=True,
                is_empty=False,
                has_git_repos=False,
                existing_repos=[],
                safe_to_proceed=False,
                message=f"Directory contains {len(contents)} items"
            )


class OnboardingService:
    """Main service orchestrating the onboarding process"""

    @staticmethod
    def initialize_state() -> OnboardingState:
        """Initialize onboarding state by loading existing configuration"""
        git_contexts = GitService.read_existing_contexts()
        existing_viewsets = ViewsetService.load_existing_viewsets()

        # Determine which contexts need setup
        contexts_to_setup = []
        contexts_already_configured = []

        for context in git_contexts:
            context_viewset_exists = False

            for viewset_name in existing_viewsets.keys():
                if (viewset_name.lower() == context.name.lower() or
                    context.name.lower() in viewset_name.lower() or
                    viewset_name.lower() in context.name.lower()):
                    context_viewset_exists = True
                    contexts_already_configured.append((context.name, viewset_name))
                    break

            if not context_viewset_exists:
                contexts_to_setup.append(context.name)

        return OnboardingState(
            git_contexts=git_contexts,
            discovered_repositories=[],  # Will be populated later if needed
            existing_viewsets=existing_viewsets,
            contexts_to_setup=contexts_to_setup,
            contexts_already_configured=contexts_already_configured
        )
