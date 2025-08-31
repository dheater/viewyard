"""Domain models for Viewyard onboarding."""

from dataclasses import dataclass
from typing import List, Dict, Optional, Any
from pathlib import Path


@dataclass
class Repository:
    """Represents a repository that can be added to a viewset"""
    name: str
    url: str
    source: str
    build_command: Optional[str] = None
    test_command: Optional[str] = None
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Repository':
        """Create Repository from dictionary"""
        return cls(
            name=data["name"],
            url=data["url"],
            source=data.get("source", "unknown"),
            build_command=data.get("build"),
            test_command=data.get("test")
        )
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert Repository to dictionary for YAML serialization"""
        result = {
            "name": self.name,
            "url": self.url
        }
        if self.build_command:
            result["build"] = self.build_command
        if self.test_command:
            result["test"] = self.test_command
        return result


@dataclass
class GitContext:
    """Represents a git context (work/personal/etc)"""
    name: str
    user_name: str
    email: str
    directory: str
    
    @property
    def config_file_path(self) -> Path:
        """Path to the git config file for this context"""
        return Path.home() / f".gitconfig-{self.name}"
    
    @property
    def source_directory(self) -> Path:
        """Source directory for this context"""
        return Path.home() / "src" / self.directory


@dataclass
class Viewset:
    """Represents a viewset configuration"""
    name: str
    context: str
    repositories: List[Repository]
    
    @classmethod
    def from_dict(cls, name: str, data: Dict[str, Any], context: str = "") -> 'Viewset':
        """Create Viewset from dictionary"""
        repos = [Repository.from_dict(repo_data) for repo_data in data.get("repos", [])]
        return cls(name=name, context=context, repositories=repos)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert Viewset to dictionary for YAML serialization"""
        return {
            "repos": [repo.to_dict() for repo in self.repositories]
        }
    
    def add_repository(self, repository: Repository) -> None:
        """Add a repository to this viewset"""
        # Avoid duplicates
        if not any(repo.name == repository.name for repo in self.repositories):
            self.repositories.append(repository)
    
    def has_repository(self, name: str) -> bool:
        """Check if viewset contains a repository with given name"""
        return any(repo.name == name for repo in self.repositories)


@dataclass
class OnboardingState:
    """Represents the current state of the onboarding process"""
    git_contexts: List[GitContext]
    discovered_repositories: List[Repository]
    existing_viewsets: Dict[str, Viewset]
    contexts_to_setup: List[str]
    contexts_already_configured: List[tuple]  # (context, viewset_name) pairs
    
    def get_context_by_name(self, name: str) -> Optional[GitContext]:
        """Get git context by name"""
        return next((ctx for ctx in self.git_contexts if ctx.name == name), None)
    
    def get_viewset_by_name(self, name: str) -> Optional[Viewset]:
        """Get viewset by name"""
        return self.existing_viewsets.get(name)
    
    def needs_setup(self) -> bool:
        """Check if any contexts need setup"""
        return len(self.contexts_to_setup) > 0
    
    def all_configured(self) -> bool:
        """Check if all contexts are already configured"""
        return len(self.contexts_to_setup) == 0


@dataclass
class RepositorySearchResult:
    """Represents search results for repositories"""
    query: str
    results: List[Repository]
    total_available: int
    
    @property
    def has_results(self) -> bool:
        """Check if search returned any results"""
        return len(self.results) > 0
    
    @property
    def is_exact_match(self) -> bool:
        """Check if there's exactly one result that matches the query exactly"""
        return (len(self.results) == 1 and 
                self.results[0].name.lower() == self.query.lower())


@dataclass
class DirectoryAnalysis:
    """Represents analysis of a viewset directory"""
    path: Path
    exists: bool
    is_empty: bool
    has_git_repos: bool
    existing_repos: List[str]
    safe_to_proceed: bool
    message: str
    
    @property
    def needs_confirmation(self) -> bool:
        """Check if user confirmation is needed before proceeding"""
        return not self.safe_to_proceed
    
    @property
    def can_import_repos(self) -> bool:
        """Check if existing git repos can be imported"""
        return self.has_git_repos and len(self.existing_repos) > 0


class OnboardingError(Exception):
    """Base exception for onboarding errors"""
    pass


class PrerequisiteError(OnboardingError):
    """Raised when prerequisites are not met"""
    pass


class GitConfigError(OnboardingError):
    """Raised when git configuration fails"""
    pass


class RepositoryDiscoveryError(OnboardingError):
    """Raised when repository discovery fails"""
    pass


class ViewsetConfigError(OnboardingError):
    """Raised when viewset configuration fails"""
    pass
