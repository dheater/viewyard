#!/usr/bin/env python3
"""Migrate a directory-based view to git-repo-based view with submodules"""

import os
import sys
import json
import shutil
import subprocess
from pathlib import Path
from typing import List, Dict, Any


def print_colored(text: str, color: str = "blue") -> None:
    """Print colored text to terminal"""
    colors = {
        "red": "\033[31m",
        "green": "\033[32m", 
        "yellow": "\033[33m",
        "blue": "\033[34m",
        "reset": "\033[0m"
    }
    print(f"{colors.get(color, '')}{text}{colors['reset']}")


def run_command(cmd: List[str], cwd: str = None, capture_output: bool = True) -> subprocess.CompletedProcess:
    """Run a command and return the result"""
    try:
        return subprocess.run(cmd, cwd=cwd, capture_output=capture_output, text=True, check=False)
    except Exception as e:
        print_colored(f"Command failed: {e}", "red")
        return subprocess.CompletedProcess(cmd, 1, "", str(e))


def load_workspace() -> List[Dict[str, Any]]:
    """Load workspace configuration"""
    workspace_file = Path("workspace.json")
    
    if not workspace_file.exists():
        print_colored("Error: workspace.json not found", "red")
        print("Run this from the viewyard directory")
        sys.exit(1)
    
    with open(workspace_file) as f:
        workspace_config = json.load(f)
    
    return workspace_config["repos"]


def get_view_repos(view_path: Path) -> List[str]:
    """Get repos in the view"""
    repos_file = view_path / ".view-repos"
    if repos_file.exists():
        with open(repos_file) as f:
            return [line.strip() for line in f if line.strip()]
    
    # Fallback: detect from directories
    return [d.name for d in view_path.iterdir() 
            if d.is_dir() and not d.name.startswith('.') and d.name != 'justfile']


def migrate_view(view_name: str) -> None:
    """Migrate a view from directory-based to git-repo-based"""
    view_path = Path("../views") / view_name
    
    if not view_path.exists():
        print_colored(f"Error: View '{view_name}' does not exist", "red")
        sys.exit(1)
    
    # Check if already migrated
    if (view_path / ".git").exists():
        print_colored(f"View '{view_name}' is already git-based", "yellow")
        return
    
    print_colored(f"Migrating view '{view_name}' to git-based architecture...", "blue")
    print("")
    
    # Get current repos in view
    current_repos = get_view_repos(view_path)
    print_colored(f"Found repos: {', '.join(current_repos)}", "blue")
    
    # Create backup
    backup_path = view_path.parent / f"{view_name}-backup"
    if backup_path.exists():
        shutil.rmtree(backup_path)
    shutil.copytree(view_path, backup_path)
    print_colored(f"Created backup at: {backup_path}", "green")
    
    # Get workspace config
    workspace_repos = load_workspace()
    
    # Store repo states (branches, uncommitted changes)
    repo_states = {}
    for repo in current_repos:
        repo_path = view_path / repo
        if repo_path.exists():
            # Get current branch
            branch_result = run_command(["git", "branch", "--show-current"], cwd=repo_path)
            current_branch = branch_result.stdout.strip() if branch_result.returncode == 0 else "master"
            
            # Check for uncommitted changes
            status_result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
            has_changes = bool(status_result.stdout.strip()) if status_result.returncode == 0 else False
            
            repo_states[repo] = {
                "branch": current_branch,
                "has_changes": has_changes
            }
    
    # Remove old repo directories
    for repo in current_repos:
        repo_path = view_path / repo
        if repo_path.exists():
            shutil.rmtree(repo_path)
    
    # Initialize git repository
    print_colored("Initializing git repository...", "blue")
    git_init_result = run_command(["git", "init"], cwd=view_path)
    if git_init_result.returncode != 0:
        print_colored(f"Failed to initialize git repository: {git_init_result.stderr}", "red")
        sys.exit(1)
    
    # Create .gitignore
    gitignore_content = """# Viewyard view repository
.view-repos
.viewyard-context
"""
    gitignore_path = view_path / ".gitignore"
    with open(gitignore_path, 'w') as f:
        f.write(gitignore_content)
    
    # Initial commit
    run_command(["git", "add", ".gitignore", "justfile"], cwd=view_path)
    run_command(["git", "commit", "-m", f"Initial commit for migrated view {view_name}"], cwd=view_path)
    
    # Create and checkout view branch
    run_command(["git", "checkout", "-b", view_name], cwd=view_path)
    
    # Add repos as submodules
    print_colored("Adding repositories as submodules...", "blue")
    for repo in current_repos:
        print_colored(f"  Adding {repo}...", "blue")
        
        # Find repo config
        repo_config = next((r for r in workspace_repos if r["name"] == repo), None)
        if not repo_config:
            print_colored(f"    Error: Repository {repo} not found in workspace.json", "red")
            continue
        
        # Add as submodule
        submodule_result = run_command(["git", "submodule", "add", repo_config["url"], repo], cwd=view_path)
        if submodule_result.returncode != 0:
            print_colored(f"    Failed to add submodule: {submodule_result.stderr}", "red")
            continue
        
        # Restore branch state
        submodule_path = view_path / repo
        repo_state = repo_states.get(repo, {})
        target_branch = repo_state.get("branch", view_name)
        
        # Try to checkout the original branch or create view branch
        branch_result = run_command(["git", "checkout", target_branch], cwd=submodule_path)
        if branch_result.returncode != 0:
            # Create new branch
            run_command(["git", "checkout", "-b", view_name], cwd=submodule_path)
        
        print_colored(f"    Added successfully on branch '{target_branch}'", "green")
    
    # Commit submodules
    run_command(["git", "add", ".gitmodules"], cwd=view_path)
    for repo in current_repos:
        run_command(["git", "add", repo], cwd=view_path)
    run_command(["git", "commit", "-m", f"Add submodules for {', '.join(current_repos)}"], cwd=view_path)
    
    print("")
    print_colored(f"Migration completed successfully!", "green")
    print_colored(f"Backup available at: {backup_path}", "yellow")
    print("")
    print_colored("Next steps:", "blue")
    print(f"  cd ../views/{view_name}")
    print("  git status                  # Verify migration")
    print("  just status                 # Check submodule status")


def main():
    """CLI interface"""
    if len(sys.argv) != 2:
        print("Usage: python3 migrate-view-to-git.py <view-name>")
        sys.exit(1)
    
    view_name = sys.argv[1]
    migrate_view(view_name)


if __name__ == "__main__":
    main()
