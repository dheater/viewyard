#!/usr/bin/env python3
"""Commands for working within a task view"""

import os
import sys
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
        print_colored(f"Error running command {' '.join(cmd)}: {e}", "red")
        return subprocess.CompletedProcess(cmd, 1, "", str(e))


def check_in_view() -> None:
    """Check if we're in a view directory"""
    # Check for git repo with submodules or viewyard context
    if not (Path(".git").exists() or Path(".viewyard-context").exists()):
        print_colored("Error: Not in a view directory", "red")
        print("Run this from within a view directory (e.g., ~/src/views/CLIENTS-469/)")
        sys.exit(1)


def get_view_repos() -> List[str]:
    """Get repos in current view"""
    check_in_view()

    # Try to get submodules first (new git-based views)
    if Path(".gitmodules").exists():
        result = run_command(["git", "submodule", "status"])
        if result.returncode == 0:
            repos = []
            for line in result.stdout.strip().split('\n'):
                if line.strip():
                    # Parse submodule status line: " hash path (tag)"
                    parts = line.strip().split()
                    if len(parts) >= 2:
                        repos.append(parts[1])
            return repos

    # Fallback to .view-repos file (legacy views)
    if Path(".view-repos").exists():
        with open(".view-repos") as f:
            return [line.strip() for line in f if line.strip()]

    return []


def get_dirty_repos() -> List[str]:
    """Get repos with uncommitted changes"""
    repos = get_view_repos()
    dirty_repos = []
    
    for repo in repos:
        repo_path = Path(repo)
        if not repo_path.exists():
            continue
        
        result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
        if result.returncode == 0 and result.stdout.strip():
            dirty_repos.append(repo)
    
    return dirty_repos


def get_ahead_repos() -> List[str]:
    """Get repos with commits ahead of upstream"""
    repos = get_view_repos()
    ahead_repos = []
    
    for repo in repos:
        repo_path = Path(repo)
        if not repo_path.exists():
            continue
        
        # Check if there are commits ahead
        result = run_command(["git", "rev-list", "@{upstream}..HEAD"], cwd=repo_path)
        if result.returncode == 0 and result.stdout.strip():
            ahead_repos.append(repo)
    
    return ahead_repos


def show_help() -> None:
    """Show help"""
    print("View Commands")
    print("=============")
    print("")
    print_colored("Commands (run from within a view directory):", "blue")
    print("  just status               - Show status of all repos in view")
    print("  just diff                 - Show diff for all repos with changes")
    print("  just rebase               - Rebase all repos against origin/master")
    print("  just build                - Build repos with uncommitted changes")
    print("  just commit-all \"msg\"     - Commit message to all dirty repos")
    print("  just push-all             - Push all repos with commits ahead")
    print("  just add-repo <repo>      - Add a repository to this view")


def view_status() -> None:
    """Show status of all repos in view"""
    repos = get_view_repos()
    
    print_colored("View Status", "blue")
    print("===========")
    print("")
    
    for repo in repos:
        print_colored(f"[{repo}]", "blue")
        
        repo_path = Path(repo)
        if not repo_path.exists():
            print_colored("  Repository not found", "red")
            continue
        
        # Get current branch and status
        branch_result = run_command(["git", "branch", "--show-current"], cwd=repo_path)
        branch = branch_result.stdout.strip() if branch_result.returncode == 0 else "unknown"
        
        status_result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
        has_changes = bool(status_result.stdout.strip()) if status_result.returncode == 0 else False
        
        ahead_result = run_command(["git", "rev-list", "@{upstream}..HEAD"], cwd=repo_path)
        ahead_count = len(ahead_result.stdout.strip().split('\n')) if ahead_result.returncode == 0 and ahead_result.stdout.strip() else 0
        
        behind_result = run_command(["git", "rev-list", "HEAD..@{upstream}"], cwd=repo_path)
        behind_count = len(behind_result.stdout.strip().split('\n')) if behind_result.returncode == 0 and behind_result.stdout.strip() else 0
        
        # Display status
        print(f"  Branch: {branch}")
        
        if has_changes:
            print_colored("  Status: Uncommitted changes", "yellow")
        else:
            print_colored("  Status: Clean", "green")
        
        if ahead_count > 0:
            print_colored(f"  Ahead: {ahead_count} commits", "yellow")
        
        if behind_count > 0:
            print_colored(f"  Behind: {behind_count} commits", "yellow")
        
        if ahead_count == 0 and behind_count == 0:
            print_colored("  Sync: Up to date", "green")
        
        print("")


def view_rebase() -> None:
    """Rebase all repos against origin/master"""
    repos = get_view_repos()

    print_colored("Rebasing All Repos Against origin/master", "blue")
    print("========================================")
    print("")

    # If this is a git-based view, update submodules first
    if Path(".gitmodules").exists():
        print_colored("Updating submodules...", "blue")
        submodule_result = run_command(["git", "submodule", "update", "--remote"])
        if submodule_result.returncode != 0:
            print_colored(f"Submodule update failed: {submodule_result.stderr}", "red")
        else:
            print_colored("Submodules updated", "green")
        print("")

    for repo in repos:
        print_colored(f"[{repo}]", "blue")

        repo_path = Path(repo)
        if not repo_path.exists():
            print_colored("  Repository not found", "red")
            continue

        # Check if repo is clean
        status_result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
        if status_result.returncode == 0 and status_result.stdout.strip():
            print_colored("  Skipping - repository has uncommitted changes", "yellow")
            continue

        # Fetch latest
        fetch_result = run_command(["git", "fetch", "origin"], cwd=repo_path)
        if fetch_result.returncode != 0:
            print_colored(f"  Fetch failed: {fetch_result.stderr}", "red")
            continue

        # Rebase against origin/master
        rebase_result = run_command(["git", "rebase", "origin/master"], cwd=repo_path)
        if rebase_result.returncode == 0:
            print_colored("  Rebased successfully", "green")
        else:
            print_colored(f"  Rebase failed: {rebase_result.stderr}", "red")

        print("")


def view_build() -> None:
    """Build repos with uncommitted changes"""
    dirty_repos = get_dirty_repos()
    
    if not dirty_repos:
        print_colored("No repos with uncommitted changes - nothing to build", "green")
        return
    
    print_colored("Building Repos with Uncommitted Changes", "blue")
    print("======================================")
    print("")
    
    # Build command mapping
    build_commands = {
        "librssconnect": ["make"],
        "universal-connection-manager": ["make"],
        "audit": ["mvn", "compile"],
        "connect": ["mvn", "compile"],
        "parent": ["mvn", "compile"],
        "go-sdk": ["go", "build", "./..."],
    }
    
    for repo in dirty_repos:
        print_colored(f"[{repo}]", "blue")
        
        repo_path = Path(repo)
        build_cmd = build_commands.get(repo, ["make"])  # Default to make
        
        print_colored(f"  Running: {' '.join(build_cmd)}", "blue")
        build_result = run_command(build_cmd, cwd=repo_path, capture_output=True)
        
        if build_result.returncode == 0:
            print_colored("  Build successful", "green")
        else:
            print_colored("  Build failed", "red")
            if build_result.stderr:
                print(f"  Error: {build_result.stderr}")
        
        print("")


def view_commit_all(message: str) -> None:
    """Commit message to all dirty repos"""
    if not message:
        print_colored("Error: Commit message is required", "red")
        sys.exit(1)
    
    dirty_repos = get_dirty_repos()
    
    if not dirty_repos:
        print_colored("No repos with uncommitted changes - nothing to commit", "green")
        return
    
    print_colored("Committing All Dirty Repos", "blue")
    print("==========================")
    print("")
    
    for repo in dirty_repos:
        print_colored(f"[{repo}]", "blue")
        
        repo_path = Path(repo)
        
        # Add all changes
        add_result = run_command(["git", "add", "."], cwd=repo_path)
        if add_result.returncode != 0:
            print_colored(f"  Add failed: {add_result.stderr}", "red")
            continue
        
        # Commit with message
        commit_result = run_command(["git", "commit", "-m", message], cwd=repo_path)
        if commit_result.returncode == 0:
            print_colored("  Committed successfully", "green")
        else:
            print_colored(f"  Commit failed: {commit_result.stderr}", "red")
        
        print("")


def view_push_all() -> None:
    """Push all repos with commits ahead"""
    ahead_repos = get_ahead_repos()

    if not ahead_repos:
        print_colored("No repos with commits ahead - nothing to push", "green")
        return

    print_colored("Pushing All Repos with Commits Ahead", "blue")
    print("====================================")
    print("")

    for repo in ahead_repos:
        print_colored(f"[{repo}]", "blue")

        repo_path = Path(repo)

        push_result = run_command(["git", "push"], cwd=repo_path)

        if push_result.returncode == 0:
            print_colored("  Pushed successfully", "green")
        else:
            print_colored("  Push failed", "red")
            if push_result.stderr:
                print(f"  Error: {push_result.stderr}")

        print("")


def view_diff() -> None:
    """Show diff for all repos in view"""
    repos = get_view_repos()

    # First collect repos with changes
    repos_with_changes = []
    for repo in repos:
        repo_path = Path(repo)
        if not repo_path.exists():
            continue

        # Check if repo has changes
        status_result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
        if status_result.returncode == 0 and status_result.stdout.strip():
            repos_with_changes.append(repo)

    if not repos_with_changes:
        print_colored("No repos with uncommitted changes", "green")
        return

    print_colored("View Diff", "blue")
    print("=========")
    print("")

    for repo in repos_with_changes:
        print_colored(f"[{repo}]", "blue")

        repo_path = Path(repo)

        # Show diff
        diff_result = run_command(["git", "diff"], cwd=repo_path, capture_output=True)

        if diff_result.returncode == 0:
            if diff_result.stdout.strip():
                print(diff_result.stdout)
            else:
                print_colored("  No diff output", "yellow")
        else:
            print_colored("  Error getting diff", "red")
            if diff_result.stderr:
                print(f"  Error: {diff_result.stderr}")

        print("")


def add_repo_to_current_view(repo_name: str) -> None:
    """Add a repository to the current view"""
    check_in_view()

    if not repo_name:
        print_colored("Error: Repository name is required", "red")
        sys.exit(1)

    # Load workspace configuration from parent directory
    import json
    workspace_file = Path("../../workspace.json")

    if not workspace_file.exists():
        print_colored("Error: workspace.json not found", "red")
        print("Make sure you're in a view directory within the viewyard workspace")
        sys.exit(1)

    with open(workspace_file) as f:
        workspace_config = json.load(f)

    available_repos = [repo["name"] for repo in workspace_config["repos"]]

    # Check if repo exists in workspace
    if repo_name not in available_repos:
        print_colored(f"Error: Repository '{repo_name}' not found in workspace", "red")
        print("Available repositories:")
        for repo in available_repos:
            print(f"  {repo}")
        sys.exit(1)

    # Check if repo directory already exists (ignore .view-repos file)
    repo_path = Path(repo_name)
    if repo_path.exists():
        print_colored(f"Repository '{repo_name}' directory already exists in this view", "yellow")
        return

    # Get current view name from directory
    current_dir = Path.cwd()
    view_name = current_dir.name

    # Add repo to .view-repos file if not already there
    existing_repos = get_view_repos()
    added_to_view_repos = False
    if repo_name not in existing_repos:
        with open(".view-repos", 'a') as f:
            f.write(f"{repo_name}\n")
        added_to_view_repos = True

    print_colored(f"Adding repository '{repo_name}' to view '{view_name}'...", "blue")

    # Find the repo config from workspace.json
    repo_config = next((r for r in workspace_config["repos"] if r["name"] == repo_name), None)

    if not repo_config:
        print_colored(f"Error: Repository {repo_name} not found in workspace.json", "red")
        return

    repo_path = Path(repo_name)

    # Clone the repository (shallow clone for speed)
    clone_cmd = ["git", "clone", "--depth", "1", repo_config["url"], str(repo_path)]
    clone_result = run_command(clone_cmd, capture_output=True)

    if clone_result.returncode == 0:
        # Create and checkout a branch named after the view
        branch_result = run_command(["git", "checkout", "-b", view_name], cwd=repo_path)
        if branch_result.returncode == 0:
            print_colored(f"Repository '{repo_name}' added successfully (shallow clone on branch '{view_name}')", "green")
        else:
            print_colored(f"Repository cloned but failed to create branch: {branch_result.stderr}", "yellow")
    else:
        print_colored(f"Failed to clone repository: {clone_result.stderr}", "red")
        # Remove from .view-repos file since clone failed (only if we added it)
        if added_to_view_repos:
            existing_repos = [line.strip() for line in open(".view-repos") if line.strip() != repo_name]
            with open(".view-repos", 'w') as f:
                for repo in existing_repos:
                    f.write(f"{repo}\n")


def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        show_help()
        return

    command = sys.argv[1]

    if command == "status":
        view_status()
    elif command == "rebase":
        view_rebase()
    elif command == "build":
        view_build()
    elif command == "commit-all":
        message = sys.argv[2] if len(sys.argv) > 2 else ""
        view_commit_all(message)
    elif command == "push-all":
        view_push_all()
    elif command == "diff":
        view_diff()
    elif command == "add-repo":
        repo_name = sys.argv[2] if len(sys.argv) > 2 else ""
        add_repo_to_current_view(repo_name)
    else:
        show_help()


if __name__ == "__main__":
    main()
