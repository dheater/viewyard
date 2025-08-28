#!/usr/bin/env python3
"""Set up simple workspace with independent repo checkouts"""

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


def main():
    """Main entry point"""
    print("Simple Workspace Setup")
    print("=====================")
    print("")
    
    # Check if we're in the right place
    workspace_file = Path("workspace.json")
    if not workspace_file.exists():
        print_colored("Error: workspace.json not found", "red")
        print("Run this from the plumbing workspace root")
        sys.exit(1)
    
    # Load workspace config
    with open(workspace_file) as f:
        workspace_config = json.load(f)
    
    workspace_info = workspace_config["workspace"]
    repos = workspace_config["repos"]
    
    print_colored(f"Setting up workspace: {workspace_info['name']}", "blue")
    print(f"Description: {workspace_info['description']}")
    print("")
    
    # Check parent directory
    parent_dir = Path("..")
    if not parent_dir.exists():
        print_colored("Error: Parent directory not found", "red")
        sys.exit(1)
    
    print_colored("Checking repository checkouts...", "blue")
    print("")
    
    for repo in repos:
        print_colored(f"[{repo['name']}]", "blue")
        
        repo_path = Path(repo["path"])
        
        if repo_path.exists():
            # Check if it's the right repository
            current_url_result = run_command(["git", "remote", "get-url", "origin"], cwd=repo_path)
            
            if current_url_result.returncode == 0:
                current_url = current_url_result.stdout.strip()
                if current_url == repo["url"]:
                    print_colored("  Already cloned and configured", "green")
                else:
                    print_colored("  Cloned but wrong URL:", "yellow")
                    print(f"    Expected: {repo['url']}")
                    print(f"    Actual:   {current_url}")
            else:
                print_colored("  Directory exists but not a git repository", "yellow")
        else:
            print_colored("  Not found - cloning...", "yellow")
            
            # Clone the repository
            clone_result = run_command(["git", "clone", repo["url"], str(repo_path)])
            if clone_result.returncode == 0:
                print_colored("  Cloned successfully", "green")
            else:
                print_colored(f"  Clone failed: {clone_result.stderr}", "red")
        
        print("")
    
    print_colored("Workspace setup complete!", "green")
    print("")
    print_colored("Next steps:", "blue")
    print("  just workspace status    # Check status of all repos")
    print("  just workspace sync      # Pull latest from all repos")
    print("  just workspace build     # Build all repos")
    print("")
    print_colored("Expected layout:", "blue")
    print("  ~/pas/")
    print("    plumbing/              # This coordination workspace")
    
    for repo in repos:
        repo_path = Path(repo["path"])
        repo_name = repo_path.name
        print(f"    {repo_name}/         # Independent checkout")


if __name__ == "__main__":
    main()
