#!/usr/bin/env python3
"""Task-based workspace view manager"""

import os
import sys
import json
import shutil
import subprocess
from pathlib import Path
from typing import List, Dict, Any


def load_workspace() -> List[Dict[str, Any]]:
    """Load workspace configuration"""
    workspace_file = Path("workspace.json")

    if not workspace_file.exists():
        print_colored("Error: workspace.json not found", "red")
        print("Run this from the viewyard workspace root")
        sys.exit(1)

    with open(workspace_file) as f:
        workspace_config = json.load(f)

    return workspace_config["repos"]


def get_available_repos() -> List[str]:
    """Get list of available repositories"""
    workspace_repos = load_workspace()
    return [repo["name"] for repo in workspace_repos]


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


def create_view_context(view_name: str, view_root: str, active_repos: List[str]) -> None:
    """Create viewyard context file for agent sandboxing"""
    import datetime

    # Convert to absolute path for consistency
    abs_view_root = str(Path(view_root).resolve())

    context = {
        "view_name": view_name,
        "view_root": abs_view_root,
        "active_repos": active_repos,
        "created": datetime.datetime.now().isoformat(),
        "workspace_boundary": {
            "allowed_paths": [
                abs_view_root,
                f"{abs_view_root}/**"
            ],
            "forbidden_paths": [
                "../**/views/*/",
                "../../views/*/"
            ]
        }
    }

    context_file = Path(view_root) / ".viewyard-context"
    with open(context_file, 'w') as f:
        json.dump(context, f, indent=2)

    print_colored(f"Created view context: {context_file}", "green")


def run_command(cmd: List[str], cwd: str = None, capture_output: bool = True) -> subprocess.CompletedProcess:
    """Run a command and return the result"""
    try:
        return subprocess.run(cmd, cwd=cwd, capture_output=capture_output, text=True, check=False)
    except Exception as e:
        print_colored(f"Error running command {' '.join(cmd)}: {e}", "red")
        return subprocess.CompletedProcess(cmd, 1, "", str(e))


def show_help() -> None:
    """Show help information"""
    print("Viewyard Task Views")
    print("==================")
    print("")
    print_colored("Commands:", "blue")
    print("  just view create <name>     # Create a new task view")
    print("  just view list              # List all views")
    print("  just view delete <name>     # Delete a view")
    print("  just view info <name>       # Show view information")
    print("  just view add-repo <view> <repo>  # Add a repo to existing view")
    print("")
    print_colored("Working in a view:", "blue")
    print("  cd ~/src/views/<view-name>")
    print("  just status                 # Show status of repos in view")
    print("  just rebase                 # Rebase repos against origin/master")
    print("  just build                  # Build repos with changes")
    print("  just commit-all \"message\"   # Commit to all dirty repos")
    print("  just push-all               # Push repos with commits ahead")


def list_views() -> None:
    """List all views"""
    views_dir = Path("../views")
    
    if not views_dir.exists():
        print_colored("No views directory found. Create your first view with: just view create <name>", "yellow")
        return
    
    views = [d.name for d in views_dir.iterdir() if d.is_dir()]
    
    if not views:
        print_colored("No views found. Create your first view with: just view create <name>", "yellow")
        return
    
    print_colored("Available Views:", "blue")
    print("===============")
    print("")
    
    for view in sorted(views):
        view_path = views_dir / view
        repos_file = view_path / ".view-repos"
        
        if repos_file.exists():
            with open(repos_file) as f:
                repos = [line.strip() for line in f if line.strip()]
            print(f"  {view} ({len(repos)} repos)")
        else:
            print(f"  {view} (no repos configured)")


def create_view(view_name: str) -> None:
    """Create a new task view"""
    if not view_name:
        print_colored("Error: View name is required", "red")
        sys.exit(1)
    
    views_dir = Path("../views")
    view_path = views_dir / view_name
    
    if view_path.exists():
        print_colored(f"Error: View '{view_name}' already exists", "red")
        sys.exit(1)
    
    # Create views directory if it doesn't exist
    views_dir.mkdir(exist_ok=True)
    
    print_colored(f"Creating view: {view_name}", "blue")
    print("")
    
    # Get available repos
    available_repos = get_available_repos()
    
    print("Available repositories:")
    for i, repo in enumerate(available_repos, 1):
        print(f"  {i}. {repo}")
    print("")
    
    # Get repo selection
    print("Select repositories for this view:")
    print("Enter numbers separated by spaces (e.g., '1 3 5'), 'all' for all repos, or 'none' to skip:")
    
    try:
        selection = input("> ").strip()
    except KeyboardInterrupt:
        print("\nCancelled.")
        sys.exit(0)
    
    selected_repos = []
    
    if selection.lower() == "all":
        selected_repos = available_repos
    elif selection.lower() != "none":
        try:
            indices = [int(x.strip()) - 1 for x in selection.split()]
            selected_repos = [available_repos[i] for i in indices if 0 <= i < len(available_repos)]
        except (ValueError, IndexError):
            print_colored("Invalid selection. Creating view with no repositories.", "yellow")
    
    # Create view directory
    view_path.mkdir(parents=True)
    
    # Create .view-repos file
    repos_file = view_path / ".view-repos"
    with open(repos_file, 'w') as f:
        for repo in selected_repos:
            f.write(f"{repo}\n")

    # Copy justfile template
    template_file = Path("templates/view-justfile")
    justfile_target = view_path / "justfile"

    if template_file.exists():
        shutil.copy2(template_file, justfile_target)

    # Initialize view as git repository
    print("")
    print_colored("Initializing view as git repository...", "blue")

    # Initialize git repo
    git_init_result = run_command(["git", "init"], cwd=view_path, capture_output=True)
    if git_init_result.returncode != 0:
        print_colored(f"Failed to initialize git repository: {git_init_result.stderr}", "red")
        sys.exit(1)

    # Create initial commit
    gitignore_content = """# Viewyard view repository
.view-repos
.viewyard-context
"""
    gitignore_path = view_path / ".gitignore"
    with open(gitignore_path, 'w') as f:
        f.write(gitignore_content)

    run_command(["git", "add", ".gitignore", "justfile"], cwd=view_path, capture_output=True)
    run_command(["git", "commit", "-m", f"Initial commit for view {view_name}"], cwd=view_path, capture_output=True)

    # Create and checkout view branch
    run_command(["git", "checkout", "-b", view_name], cwd=view_path, capture_output=True)

    # Add selected repositories as submodules
    if selected_repos:
        print("")
        print_colored("Adding repositories as submodules...", "blue")
        print("")

        workspace_repos = load_workspace()

        for repo in selected_repos:
            print_colored(f"[{repo}]", "blue")

            # Find the repo config from workspace.json
            repo_config = next((r for r in workspace_repos if r["name"] == repo), None)

            if not repo_config:
                print_colored(f"  Error: Repository {repo} not found in workspace.json", "red")
                continue

            # Add as submodule
            submodule_cmd = ["git", "submodule", "add", repo_config["url"], repo]
            submodule_result = run_command(submodule_cmd, cwd=view_path, capture_output=True)

            if submodule_result.returncode == 0:
                # Create and checkout a branch named after the view in the submodule
                submodule_path = view_path / repo
                branch_result = run_command(["git", "checkout", "-b", view_name], cwd=submodule_path, capture_output=True)
                if branch_result.returncode == 0:
                    print_colored(f"  Added as submodule on branch '{view_name}'", "green")
                else:
                    print_colored(f"  Added as submodule but failed to create branch: {branch_result.stderr}", "yellow")
            else:
                print_colored(f"  Failed to add submodule: {submodule_result.stderr}", "red")

        # Commit the submodule additions
        run_command(["git", "add", ".gitmodules"], cwd=view_path, capture_output=True)
        for repo in selected_repos:
            run_command(["git", "add", repo], cwd=view_path, capture_output=True)
        run_command(["git", "commit", "-m", f"Add submodules for {', '.join(selected_repos)}"], cwd=view_path, capture_output=True)

    # Create viewyard context file for agent sandboxing (after git init)
    create_view_context(view_name, str(view_path), selected_repos)

    print("")
    print_colored(f"View '{view_name}' created successfully as git repository!", "green")
    print("")
    print_colored("Next steps:", "blue")
    print(f"  cd ../views/{view_name}")
    print("  just status                 # Check status of submodules in view")
    print("  just rebase                 # Rebase submodules against origin/master")


def delete_view(view_name: str) -> None:
    """Delete a view"""
    if not view_name:
        print_colored("Error: View name is required", "red")
        sys.exit(1)
    
    view_path = Path("../views") / view_name

    if not view_path.exists():
        print_colored(f"Error: View '{view_name}' does not exist", "red")
        sys.exit(1)
    
    print_colored(f"Delete view '{view_name}'? This cannot be undone.", "yellow")
    try:
        confirm = input("Type 'yes' to confirm: ").strip()
    except KeyboardInterrupt:
        print("\nCancelled.")
        sys.exit(0)
    
    if confirm.lower() == "yes":
        shutil.rmtree(view_path)
        print_colored(f"View '{view_name}' deleted successfully!", "green")
    else:
        print("Cancelled.")


def show_view_info(view_name: str) -> None:
    """Show information about a view"""
    if not view_name:
        print_colored("Error: View name is required", "red")
        sys.exit(1)
    
    view_path = Path("../views") / view_name

    if not view_path.exists():
        print_colored(f"Error: View '{view_name}' does not exist", "red")
        sys.exit(1)
    
    repos_file = view_path / ".view-repos"
    
    print_colored(f"View: {view_name}", "blue")
    print("=" * (len(view_name) + 6))
    print("")
    
    if repos_file.exists():
        with open(repos_file) as f:
            repos = [line.strip() for line in f if line.strip()]
        
        print_colored("Repositories:", "blue")
        for repo in repos:
            repo_path = Path("..") / repo
            if repo_path.exists():
                print(f"  ✓ {repo}")
            else:
                print_colored(f"  ✗ {repo} (not found)", "red")
    else:
        print_colored("No repositories configured", "yellow")


def add_repo_to_view(view_name: str, repo_name: str) -> None:
    """Add a repository to an existing view"""
    if not view_name:
        print_colored("Error: View name is required", "red")
        sys.exit(1)

    if not repo_name:
        print_colored("Error: Repository name is required", "red")
        sys.exit(1)

    view_path = Path("../views") / view_name

    if not view_path.exists():
        print_colored(f"Error: View '{view_name}' does not exist", "red")
        sys.exit(1)

    # Check if repo exists in workspace
    available_repos = get_available_repos()
    if repo_name not in available_repos:
        print_colored(f"Error: Repository '{repo_name}' not found in workspace", "red")
        print("Available repositories:")
        for repo in available_repos:
            print(f"  {repo}")
        sys.exit(1)

    repos_file = view_path / ".view-repos"

    # Check if repo is already in view
    existing_repos = []
    if repos_file.exists():
        with open(repos_file) as f:
            existing_repos = [line.strip() for line in f if line.strip()]

    if repo_name in existing_repos:
        print_colored(f"Repository '{repo_name}' is already in view '{view_name}'", "yellow")
        return

    # Add repo to .view-repos file
    with open(repos_file, 'a') as f:
        f.write(f"{repo_name}\n")

    print_colored(f"Adding repository '{repo_name}' to view '{view_name}'...", "blue")

    # Find the repo config from workspace.json
    workspace_repos = load_workspace()
    repo_config = next((r for r in workspace_repos if r["name"] == repo_name), None)

    if not repo_config:
        print_colored(f"Error: Repository {repo_name} not found in workspace.json", "red")
        return

    repo_path = view_path / repo_name

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
        # Remove from .view-repos file since clone failed
        existing_repos = [line.strip() for line in open(repos_file) if line.strip() != repo_name]
        with open(repos_file, 'w') as f:
            for repo in existing_repos:
                f.write(f"{repo}\n")


def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        show_help()
        return
    
    command = sys.argv[1]
    
    if command == "create":
        view_name = sys.argv[2] if len(sys.argv) > 2 else ""
        create_view(view_name)
    elif command == "list":
        list_views()
    elif command == "delete":
        view_name = sys.argv[2] if len(sys.argv) > 2 else ""
        delete_view(view_name)
    elif command == "info":
        view_name = sys.argv[2] if len(sys.argv) > 2 else ""
        show_view_info(view_name)
    elif command == "add-repo":
        view_name = sys.argv[2] if len(sys.argv) > 2 else ""
        repo_name = sys.argv[3] if len(sys.argv) > 3 else ""
        add_repo_to_view(view_name, repo_name)
    else:
        show_help()


if __name__ == "__main__":
    main()
