#!/usr/bin/env python3
"""Clean, modular Viewyard onboarding script."""

import sys
from pathlib import Path

# Import our clean modules
from models import Repository, OnboardingState
from services import (
    PrerequisiteService, GitService, GitHubService, RepositoryService,
    ViewsetService, OnboardingService
)
from ui import UI, OnboardingUI


class OnboardingController:
    """Main controller orchestrating the onboarding process"""
    
    def __init__(self):
        self.state: OnboardingState = None
    
    def run(self) -> None:
        """Run the complete onboarding process"""
        try:
            # Step 1: Welcome and prerequisites
            OnboardingUI.show_welcome()
            self._check_prerequisites()
            
            # Step 2: Initialize state
            self.state = OnboardingService.initialize_state()
            
            # Step 3: Show git contexts
            OnboardingUI.show_git_contexts(self.state.git_contexts)
            
            # Step 4: Show onboarding state
            OnboardingUI.show_onboarding_state(self.state)
            
            # Step 5: Handle existing viewsets or setup new ones
            if self.state.all_configured():
                self._handle_all_configured()
            else:
                self._setup_new_viewsets()
            
            # Step 6: Final message
            OnboardingUI.show_final_message()
            
        except KeyboardInterrupt:
            print("\n\nOnboarding cancelled by user.")
            sys.exit(1)
        except Exception as e:
            UI.print_error(f"Onboarding failed: {e}")
            sys.exit(1)
    
    def _check_prerequisites(self) -> None:
        """Check and validate prerequisites"""
        all_good = (PrerequisiteService.check_all() and 
                   PrerequisiteService.check_python_packages())
        OnboardingUI.show_prerequisites_status(all_good)
    
    def _handle_all_configured(self) -> None:
        """Handle case where all contexts are already configured"""
        if not self.state.contexts_already_configured:
            return
        
        # Ask if user wants to update existing viewsets
        update_existing = UI.ask_yes_no(
            "Would you like to discover and add new repositories to existing viewsets?"
        )
        
        if update_existing:
            self._update_existing_viewsets()
    
    def _update_existing_viewsets(self) -> None:
        """Update existing viewsets with newly discovered repositories"""
        print("")
        print("🔍 Discovering repositories to update existing viewsets...")
        
        # Discover repositories
        discovered_repos = GitHubService.discover_all_repositories()
        accounts = GitHubService.get_available_accounts()
        OnboardingUI.show_repository_discovery_status(len(discovered_repos), accounts)
        
        # Update each existing viewset
        for context, viewset_name in self.state.contexts_already_configured:
            self._update_single_viewset(context, viewset_name, discovered_repos)
        
        UI.print_success("Viewset updates complete!")
    
    def _update_single_viewset(self, context: str, viewset_name: str, 
                              discovered_repos: list) -> None:
        """Update a single viewset with new repositories"""
        print(f"\n{'='*60}")
        UI.print_colored(f"Updating viewset '{viewset_name}' ({context} context):", "blue")
        
        # Filter repos for this context
        context_repos = RepositoryService.filter_by_context(discovered_repos, context)
        existing_viewset = self.state.existing_viewsets[viewset_name]
        existing_repo_names = {repo.name for repo in existing_viewset.repositories}
        new_repos = [repo for repo in context_repos if repo.name not in existing_repo_names]
        
        if new_repos:
            print(f"Found {len(new_repos)} new repositories for {viewset_name}:")
            for repo in new_repos[:5]:  # Show first 5
                print(f"  • {repo.name} ({repo.source})")
            if len(new_repos) > 5:
                print(f"  ... and {len(new_repos) - 5} more")
            
            add_new = UI.ask_yes_no(f"Add these {len(new_repos)} repositories to {viewset_name}?")
            if add_new:
                # Add new repos to existing viewset
                for repo in new_repos:
                    existing_viewset.add_repository(repo)
                
                # Save updated config
                ViewsetService.save_viewsets(self.state.existing_viewsets)
                UI.print_success(f"Added {len(new_repos)} repositories to {viewset_name}")
        else:
            print(f"No new repositories found for {viewset_name}")
    
    def _setup_new_viewsets(self) -> None:
        """Setup new viewsets for contexts that need them"""
        # Discover repositories once
        discovered_repos = GitHubService.discover_all_repositories()
        accounts = GitHubService.get_available_accounts()
        OnboardingUI.show_repository_discovery_status(len(discovered_repos), accounts)
        
        # Setup each context that needs it
        for context in self.state.contexts_to_setup:
            self._setup_single_viewset(context, discovered_repos)
    
    def _setup_single_viewset(self, context: str, discovered_repos: list) -> None:
        """Setup a single viewset for a context"""
        # Filter repositories for this context
        context_repos = RepositoryService.filter_by_context(discovered_repos, context)
        
        # Get viewset name from user
        viewset_name = OnboardingUI.show_viewset_setup_header(
            context, context, len(context_repos)
        )
        
        if len(context_repos) == 0:
            self._handle_no_repositories(context, viewset_name, discovered_repos)
            return
        
        # Show selection options
        OnboardingUI.show_repository_selection_options()
        
        # Repository selection loop
        selected_repos = []
        while True:
            OnboardingUI.show_selected_repositories(selected_repos)
            
            query = input("Search repositories (or 'done'/'manual'/'*'): ").strip()
            
            if query.lower() == 'done':
                break
            elif query == '*':
                # Add all repositories
                for repo in context_repos:
                    if not any(r.name == repo.name for r in selected_repos):
                        selected_repos.append(repo)
                UI.print_success(f"Added all {len(context_repos)} available repositories")
                continue
            elif query.lower() == 'manual':
                self._handle_manual_mode(selected_repos, discovered_repos)
                continue
            elif not query:
                # Show all repositories
                self._show_all_repositories(context_repos)
                continue
            
            # Search for repositories
            search_result = RepositoryService.fuzzy_search(context_repos, query)
            selected_repo = OnboardingUI.show_search_results(search_result)
            
            if selected_repo and not any(r.name == selected_repo.name for r in selected_repos):
                selected_repos.append(selected_repo)
                UI.print_success(f"Added {selected_repo.name}")
        
        # Create and save viewset
        if selected_repos:
            viewset = Viewset(name=viewset_name, context=context, repositories=selected_repos)
            self.state.existing_viewsets[viewset_name] = viewset
            ViewsetService.save_viewsets(self.state.existing_viewsets)
            OnboardingUI.show_completion_message(viewset_name)
    
    def _handle_no_repositories(self, context: str, viewset_name: str, 
                               discovered_repos: list) -> None:
        """Handle case where no repositories found for context"""
        # Provide helpful guidance
        current_auth = self._get_current_github_user()
        
        if context.lower() in ['personal', 'dheater'] and current_auth == 'daniel-heater-imprivata':
            print("This is because you're authenticated with your work GitHub account.")
            print("To discover personal repositories:")
            print("  1. Run: gh auth switch --user dheater")
            print("  2. Re-run this onboarding script")
            print("  3. Or manually add personal repositories below")
        elif context.lower() in ['work', 'imprivata'] and current_auth == 'dheater':
            print("This is because you're authenticated with your personal GitHub account.")
            print("To discover work repositories:")
            print("  1. Run: gh auth switch --user daniel-heater-imprivata")
            print("  2. Re-run this onboarding script")
            print("  3. Or manually add work repositories below")
        
        print("")
        UI.print_colored(f"Let's add repositories manually to your '{viewset_name}' viewset.", "blue")
        
        # Manual repository entry
        selected_repos = []
        self._handle_manual_mode(selected_repos, discovered_repos)
        
        # Create viewset even if empty
        viewset = Viewset(name=viewset_name, context=context, repositories=selected_repos)
        self.state.existing_viewsets[viewset_name] = viewset
        ViewsetService.save_viewsets(self.state.existing_viewsets)
        OnboardingUI.show_completion_message(viewset_name)
    
    def _handle_manual_mode(self, selected_repos: list, discovered_repos: list) -> None:
        """Handle manual repository entry mode"""
        print("Manual Repository Entry:")
        print("You can:")
        print("• Type a repository name to search through all discovered repositories")
        print("• Type '*' to add all discovered repositories")
        print("• Type 'search' to return to search mode")
        print("• Type 'done' when finished")
        print("• Or enter a repository name and URL manually")
        print("")
        
        while True:
            repo_name = input("Repository name (or press Enter to return to search): ").strip()
            if not repo_name:
                break
            
            if repo_name == '*':
                # Add all discovered repositories
                added_count = 0
                for repo in discovered_repos:
                    if not any(r.name == repo.name for r in selected_repos):
                        selected_repos.append(repo)
                        added_count += 1
                UI.print_success(f"Added {added_count} repositories")
                continue
            elif repo_name.lower() == 'done':
                break
            elif repo_name.lower() == 'search':
                break
            
            # Try to search for the repository
            search_result = RepositoryService.fuzzy_search(discovered_repos, repo_name)
            if search_result.has_results:
                selected_repo = OnboardingUI.show_search_results(search_result)
                if selected_repo and not any(r.name == selected_repo.name for r in selected_repos):
                    selected_repos.append(selected_repo)
                    UI.print_success(f"Added {selected_repo.name}")
                continue
            
            # Manual URL entry
            repo_url = input(f"Git URL for {repo_name}: ").strip()
            if repo_url:
                manual_repo = Repository(name=repo_name, url=repo_url, source="manual")
                selected_repos.append(manual_repo)
                UI.print_success(f"Added {repo_name}")
            else:
                print("Skipping - URL is required")
    
    def _show_all_repositories(self, repositories: list) -> None:
        """Show all available repositories"""
        print("")
        print(f"All available repositories ({len(repositories)}):")
        for i, repo in enumerate(repositories, 1):
            print(f"   {i}. {repo.name} ({repo.source})")
            if i >= 20:  # Limit display
                print(f"   ... and {len(repositories) - 20} more")
                break
        print("")
    
    def _get_current_github_user(self) -> str:
        """Get current GitHub user"""
        try:
            import subprocess
            result = subprocess.run(["gh", "api", "user", "--jq", ".login"],
                                  capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                return result.stdout.strip()
        except:
            pass
        return "unknown"


def main():
    """Main entry point"""
    controller = OnboardingController()
    controller.run()


if __name__ == "__main__":
    main()
