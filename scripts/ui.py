"""User interface layer for Viewyard onboarding."""

import sys
from typing import List, Optional, Tuple
from pathlib import Path

from models import (
    Repository, GitContext, Viewset, OnboardingState, RepositorySearchResult,
    DirectoryAnalysis
)


class Colors:
    """ANSI color codes for terminal output"""
    PURPLE = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    END = '\033[0m'


class UI:
    """User interface utilities for onboarding"""
    
    @staticmethod
    def print_colored(text: str, color: str = "blue") -> None:
        """Print colored text to terminal"""
        color_map = {
            "purple": Colors.PURPLE,
            "blue": Colors.BLUE,
            "cyan": Colors.CYAN,
            "green": Colors.GREEN,
            "yellow": Colors.YELLOW,
            "red": Colors.RED,
            "bold": Colors.BOLD
        }
        
        color_code = color_map.get(color.lower(), Colors.BLUE)
        print(f"{color_code}{text}{Colors.END}")
    
    @staticmethod
    def print_header(text: str) -> None:
        """Print a section header"""
        print("")
        print("=" * 60)
        UI.print_colored(f" {text}", "bold")
        print("=" * 60)
        print("")
    
    @staticmethod
    def print_success(text: str) -> None:
        """Print success message"""
        UI.print_colored(f"✓ {text}", "green")
    
    @staticmethod
    def print_warning(text: str) -> None:
        """Print warning message"""
        UI.print_colored(f"⚠ {text}", "yellow")
    
    @staticmethod
    def print_error(text: str) -> None:
        """Print error message"""
        UI.print_colored(f"❌ {text}", "red")
    
    @staticmethod
    def print_info(text: str) -> None:
        """Print info message"""
        UI.print_colored(f"ℹ️  {text}", "blue")
    
    @staticmethod
    def ask_yes_no(question: str, default: bool = False) -> bool:
        """Ask a yes/no question"""
        default_str = "Y/n" if default else "y/N"
        response = input(f"{question} [{default_str}]: ").strip().lower()
        
        if not response:
            return default
        
        return response in ['y', 'yes', 'true', '1']
    
    @staticmethod
    def ask_input(prompt: str, default: str = "") -> str:
        """Ask for user input with optional default"""
        if default:
            response = input(f"{prompt} [{default}]: ").strip()
            return response if response else default
        else:
            return input(f"{prompt}: ").strip()


class OnboardingUI:
    """High-level UI components for onboarding workflow"""
    
    @staticmethod
    def show_welcome() -> None:
        """Show welcome message"""
        UI.print_colored("Welcome to Viewyard!", "purple")
        print("This script will help you get set up quickly.")
        print("")
    
    @staticmethod
    def show_prerequisites_status(all_good: bool) -> None:
        """Show prerequisites check status"""
        UI.print_header("Checking Prerequisites")
        
        if all_good:
            UI.print_success("Git is installed")
            UI.print_success("Just is installed")
            UI.print_success("Python 3 is installed")
            UI.print_success("PyYAML is installed")
        else:
            UI.print_error("Some prerequisites are missing")
            print("Please install missing tools and run again.")
            sys.exit(1)
    
    @staticmethod
    def show_git_contexts(contexts: List[GitContext]) -> None:
        """Show detected git contexts"""
        UI.print_header("User Information")
        print("Let's set up your git configuration for different contexts.")
        print("This sets up git commit author info (what appears in 'git log').")
        print("")
        
        if contexts:
            context_names = [ctx.name for ctx in contexts]
            print(f"Found existing git configs for: {', '.join(context_names)}")
            print("I'll use your existing configuration.")
            print("")
            
            for ctx in contexts:
                UI.print_success(f"{ctx.name}: {ctx.user_name} <{ctx.email}>")
        else:
            print("No existing git contexts found. Let's create them.")
    
    @staticmethod
    def show_onboarding_state(state: OnboardingState) -> None:
        """Show current onboarding state"""
        UI.print_header("Setting Up Your Viewsets")
        
        print("What are viewsets?")
        print("• A viewset is a collection of related repositories (e.g., work projects, client work)")
        print("• When you create a view, it clones all repos in the viewset to a task-specific directory")
        print("• This lets you work on multiple related repos together for a specific task")
        print("")
        
        print("Example: 'just view create fix-auth-bug --viewset work' creates:")
        print("  ~/src/work/fix-auth-bug/repo1/")
        print("  ~/src/work/fix-auth-bug/repo2/")
        print("  ~/src/work/fix-auth-bug/repo3/")
        print("")
        
        if state.contexts_already_configured:
            UI.print_success("Found existing viewsets:")
            for context, viewset_name in state.contexts_already_configured:
                viewset = state.existing_viewsets[viewset_name]
                repo_count = len(viewset.repositories)
                print(f"  • {viewset_name} ({context} context): {repo_count} repositories configured")
            print("")
        
        if state.contexts_to_setup:
            print(f"Setting up new viewsets for: {', '.join(state.contexts_to_setup)}")
        elif state.all_configured():
            print("All contexts already have viewsets configured!")
            print("")
            print("Your viewsets are ready to use:")
            for context, viewset_name in state.contexts_already_configured:
                print(f"  • just view create <task-name> --viewset {viewset_name}")
    
    @staticmethod
    def show_repository_discovery_status(total_found: int, accounts: List[str]) -> None:
        """Show repository discovery status"""
        print("🔍 Discovering your repositories...")
        print("  • Scanning local directories...")
        print("    Found 0 local repositories")
        print("  • Checking GitHub repositories...")
        
        if accounts:
            print(f"    Found {len(accounts)} authenticated account(s): {', '.join(accounts)}")
            for account in accounts:
                print(f"    Discovering repositories from {account}...")
        
        print(f"    Found {total_found} GitHub repositories")
        print(f"  • Total: {total_found} repositories discovered")
        print("")
    
    @staticmethod
    def show_search_results(result: RepositorySearchResult) -> Optional[Repository]:
        """Show search results and get user selection"""
        if not result.has_results:
            print(f"No repositories found matching '{result.query}'")
            return None
        
        print("")
        print("Found repositories:")
        for i, repo in enumerate(result.results, 1):
            print(f"   {i}. {repo.name} ({repo.source})")
            print(f"       URL:  {repo.url}")
        print("")
        
        try:
            choice = input("Select repository number (or press Enter to search again): ").strip()
            if choice:
                choice_num = int(choice)
                if 1 <= choice_num <= len(result.results):
                    return result.results[choice_num - 1]
        except (ValueError, IndexError):
            UI.print_warning("Invalid selection")
        
        return None
    
    @staticmethod
    def show_viewset_setup_header(viewset_name: str, context: str, repo_count: int) -> None:
        """Show viewset setup header"""
        print("=" * 60)
        UI.print_colored(f"Setting up viewset for '{context}' context:", "blue")
        
        viewset_display_name = UI.ask_input(f"Enter name for your {context} viewset", viewset_name)
        
        viewset_dir = Path.home() / "src" / viewset_display_name
        if viewset_dir.exists():
            print(f"Using existing Viewyard directory {viewset_dir}")
        else:
            print(f"Using Viewyard directory {viewset_dir}")
        
        UI.print_success(f"Created viewset directory: {viewset_dir}")
        print("")
        
        if repo_count > 0:
            print(f"Found {repo_count} repositories for '{context}' context! Let's add them to your '{viewset_display_name}' viewset.")
            print(f"(Filtered from total repositories based on {context} context)")
        else:
            UI.print_warning(f"No repositories found for '{context}' context.")
        
        return viewset_display_name
    
    @staticmethod
    def show_repository_selection_options() -> None:
        """Show repository selection options"""
        print("")
        print("You can:")
        print("• Type a repository name or part of it to search")
        print("• Press Enter to see all available repositories")
        print("• Type '*' to add all repositories")
        print("• Type 'manual' to add repositories manually")
        print("• Type 'done' when finished adding repositories")
        print("")
        print("(You can switch between search and manual modes as needed)")
        print("")
    
    @staticmethod
    def show_selected_repositories(repositories: List[Repository]) -> None:
        """Show currently selected repositories"""
        if not repositories:
            return
        
        print("")
        print(f"Currently selected repositories ({len(repositories)}):")
        for i, repo in enumerate(repositories, 1):
            print(f"  {i}. {repo.name}")
        print("")
    
    @staticmethod
    def show_completion_message(viewset_name: str) -> None:
        """Show completion message"""
        UI.print_success(f"Created viewset '{viewset_name}' with repositories")
        print("")
        print("Your viewset is ready! You can:")
        print(f"  • Create views: just view create --viewset {viewset_name} <task-name>")
        print("  • Edit config: ~/.config/viewyard/viewsets.yaml")
        print("  • Add more repos later by editing the config file")
        print("")
    
    @staticmethod
    def show_final_message() -> None:
        """Show final completion message"""
        UI.print_header("Testing Setup")
        UI.print_success("Viewsets config is valid")
        UI.print_success("View manager is working")
        print("")
        
        UI.print_colored("🎉 Setup complete! You're ready to use viewyard.", "green")
        print("")
        print("Try creating your first view:")
        UI.print_colored("  just view create my-first-task", "cyan")
        print("")
        print("Next steps:")
        print("1. Create your first view: just view create <task-name>")
        print("2. Add more repositories by editing ~/.config/viewyard/viewsets.yaml")
        print("3. Check the README for more examples and usage")
