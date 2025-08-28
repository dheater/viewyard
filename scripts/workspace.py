#!/usr/bin/env python3
"""Simple workspace coordination - no Git magic, just scripts"""

import os
import sys
import json
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


def get_existing_repos() -> List[Dict[str, Any]]:
    """Get list of repos that actually exist"""
    repos = load_workspace()
    return [repo for repo in repos if Path(repo["path"]).exists()]


def show_help() -> None:
    """Show help"""
    print("Viewyard Workspace Coordination")
    print("===============================")
    print("")
    print_colored("Commands:", "blue")
    print("  just workspace status     - Show git status of all repos")
    print("  just workspace sync       - Pull latest from all repos")
    print("  just workspace build      - Build all repos")
    print("  just workspace test       - Test all repos")
    print("  just workspace branch <name> - Create branch in all repos")
    print("")
    print_colored("Repo Layout:", "blue")
    print("  ~/src/")
    print("    viewyard/              # This coordination workspace")
    print("    librssconnect/         # Independent repo checkout")
    print("    universal-connection-manager/  # Independent repo checkout")
    print("    audit/                 # Independent repo checkout")
    print("    ...")


def workspace_status() -> None:
    """Show status of all repos"""
    print_colored("Workspace Status", "blue")
    print("================")
    print("")
    
    repos = get_existing_repos()
    
    for repo in repos:
        print_colored(f"[{repo['name']}]", "blue")
        
        repo_path = Path(repo["path"])
        
        # Get current branch and status
        branch_result = run_command(["git", "branch", "--show-current"], cwd=repo_path)
        branch = branch_result.stdout.strip() if branch_result.returncode == 0 else "unknown"
        
        status_result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
        
        if status_result.returncode == 0:
            if not status_result.stdout.strip():
                print_colored(f"  Clean - on {branch}", "green")
            else:
                print_colored(f"  Modified - on {branch}", "yellow")
                for line in status_result.stdout.strip().split('\n'):
                    if line.strip():
                        print(f"    {line}")
        else:
            print_colored(f"  Error getting status - on {branch}", "red")
        
        print("")


def workspace_sync() -> None:
    """Sync all repos (pull latest)"""
    print_colored("Syncing All Repositories", "blue")
    print("========================")
    print("")
    
    repos = get_existing_repos()
    
    for repo in repos:
        print_colored(f"[{repo['name']}]", "blue")
        
        repo_path = Path(repo["path"])
        
        # Check if repo is clean
        status_result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
        if status_result.returncode == 0 and status_result.stdout.strip():
            print_colored("  Skipping - repository has uncommitted changes", "yellow")
            continue
        
        # Pull latest
        pull_result = run_command(["git", "pull"], cwd=repo_path)
        if pull_result.returncode == 0:
            print_colored("  Synced successfully", "green")
        else:
            print_colored(f"  Sync failed: {pull_result.stderr}", "red")
        
        print("")


def workspace_build() -> None:
    """Build all repos"""
    print_colored("Building All Repositories", "blue")
    print("=========================")
    print("")
    
    repos = get_existing_repos()
    
    for repo in repos:
        print_colored(f"[{repo['name']}]", "blue")
        
        repo_path = Path(repo["path"])
        build_cmd = repo["build"].split()
        
        # Run build command
        build_result = run_command(build_cmd, cwd=repo_path)
        if build_result.returncode == 0:
            print_colored("  Build successful", "green")
        else:
            print_colored("  Build failed", "red")
            if build_result.stderr:
                print(f"  Error: {build_result.stderr}")
        
        print("")


def workspace_test() -> None:
    """Test all repos"""
    print_colored("Testing All Repositories", "blue")
    print("========================")
    print("")
    
    repos = get_existing_repos()
    
    for repo in repos:
        print_colored(f"[{repo['name']}]", "blue")
        
        repo_path = Path(repo["path"])
        test_cmd = repo["test"].split()
        
        # Run test command
        test_result = run_command(test_cmd, cwd=repo_path)
        if test_result.returncode == 0:
            print_colored("  Tests passed", "green")
        else:
            print_colored("  Tests failed", "red")
            if test_result.stderr:
                print(f"  Error: {test_result.stderr}")
        
        print("")


def workspace_branch(branch_name: str) -> None:
    """Create branch in all repos"""
    if not branch_name:
        print_colored("Usage: just workspace branch <branch-name>", "yellow")
        return
    
    print_colored(f"Creating Branch '{branch_name}' in All Repositories", "blue")
    print("==================================================")
    print("")
    
    repos = get_existing_repos()
    
    for repo in repos:
        print_colored(f"[{repo['name']}]", "blue")
        
        repo_path = Path(repo["path"])
        
        # Check if repo is clean
        status_result = run_command(["git", "status", "--porcelain"], cwd=repo_path)
        if status_result.returncode == 0 and status_result.stdout.strip():
            print_colored("  Skipping - repository has uncommitted changes", "yellow")
            continue
        
        # Check if branch already exists
        branch_check = run_command(["git", "branch", "--list", branch_name], cwd=repo_path)
        if branch_check.returncode == 0 and branch_check.stdout.strip():
            print_colored("  Branch already exists, checking out", "yellow")
            run_command(["git", "checkout", branch_name], cwd=repo_path, capture_output=False)
        else:
            print_colored("  Creating new branch", "green")
            run_command(["git", "checkout", "-b", branch_name], cwd=repo_path, capture_output=False)
        
        print("")


def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        show_help()
        return
    
    command = sys.argv[1]
    
    if command == "status":
        workspace_status()
    elif command == "sync":
        workspace_sync()
    elif command == "build":
        workspace_build()
    elif command == "test":
        workspace_test()
    elif command == "branch":
        branch_name = sys.argv[2] if len(sys.argv) > 2 else ""
        workspace_branch(branch_name)
    else:
        show_help()


if __name__ == "__main__":
    main()
