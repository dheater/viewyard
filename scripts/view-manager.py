#!/usr/bin/env python3
"""Task-based workspace view manager"""

import os
import sys
import json
import shutil
import subprocess
import yaml
from pathlib import Path
from typing import List, Dict, Any, Optional


def load_viewsets_config() -> Dict[str, Any]:
    """Load viewsets configuration from ~/.config/viewyard/viewsets.yaml"""
    config_dir = Path.home() / ".config" / "viewyard"
    config_file = config_dir / "viewsets.yaml"

    if not config_file.exists():
        print_colored("Error: viewsets.yaml not found", "red")
        print(f"Expected location: {config_file}")
        print("Create a viewsets.yaml file with your repository configurations")
        sys.exit(1)

    try:
        with open(config_file) as f:
            config = yaml.safe_load(f)
        return config
    except yaml.YAMLError as e:
        print_colored(f"Error parsing viewsets.yaml: {e}", "red")
        sys.exit(1)


def load_workspace(viewset_name: Optional[str] = None) -> List[Dict[str, Any]]:
    """Load workspace configuration for a specific viewset"""
    config = load_viewsets_config()

    if "viewsets" not in config:
        print_colored("Error: 'viewsets' key not found in viewsets.yaml", "red")
        sys.exit(1)

    viewsets = config["viewsets"]

    if not viewsets:
        print_colored("Error: No viewsets defined in viewsets.yaml", "red")
        sys.exit(1)

    # Use specified viewset, auto-detected viewset, or default to first one
    if viewset_name:
        if viewset_name not in viewsets:
            print_colored(f"Error: Viewset '{viewset_name}' not found", "red")
            available = ", ".join(viewsets.keys())
            print(f"Available viewsets: {available}")
            sys.exit(1)
        selected_viewset = viewsets[viewset_name]
    else:
        # Try to auto-detect viewset from current directory
        detected_viewset = detect_current_viewset()
        if detected_viewset:
            viewset_name = detected_viewset
            selected_viewset = viewsets[viewset_name]
            print_colored(f"Auto-detected viewset: {viewset_name}", "blue")
        else:
            # Default to first viewset
            viewset_name = next(iter(viewsets))
            selected_viewset = viewsets[viewset_name]
            print_colored(f"Using default viewset: {viewset_name}", "blue")

    if "repos" not in selected_viewset:
        print_colored(f"Error: No 'repos' defined in viewset '{viewset_name}'", "red")
        sys.exit(1)

    return selected_viewset["repos"]


def get_available_repos(viewset_name: Optional[str] = None) -> List[str]:
    """Get list of available repositories"""
    workspace_repos = load_workspace(viewset_name)
    return [repo["name"] for repo in workspace_repos]


def detect_current_viewset() -> Optional[str]:
    """Detect current viewset from working directory path"""
    cwd = Path.cwd().resolve()

    # Check if we're in a viewset directory structure: ~/src/<viewset>/views/ or ~/src/<viewset>/
    src_path = Path.home().resolve() / "src"

    try:
        # Get relative path from ~/src/
        rel_path = cwd.relative_to(src_path)
        path_parts = rel_path.parts

        if len(path_parts) >= 1:
            potential_viewset = path_parts[0]

            # Validate this is actually a configured viewset
            config = load_viewsets_config()
            viewsets = config.get("viewsets", {})

            if potential_viewset in viewsets:
                return potential_viewset

    except ValueError:
        # Not under ~/src/, so no viewset detected
        pass

    return None


def get_viewset_from_args(args: List[str]) -> tuple[Optional[str], List[str]]:
    """Extract viewset from arguments and return (viewset_name, remaining_args)"""
    viewset_name = None
    remaining_args = []

    i = 0
    while i < len(args):
        if args[i] == "--viewset" and i + 1 < len(args):
            viewset_name = args[i + 1]
            i += 2  # Skip both --viewset and its value
        else:
            remaining_args.append(args[i])
            i += 1

    return viewset_name, remaining_args


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


def ensure_viewset_justfiles() -> None:
    """Ensure all configured viewsets have justfiles in their directories"""
    config = load_viewsets_config()
    viewsets = config.get("viewsets", {})

    for viewset_name in viewsets.keys():
        viewset_dir = Path.home() / "src" / viewset_name
        justfile_path = viewset_dir / "justfile"

        # Create viewset directory if it doesn't exist
        viewset_dir.mkdir(parents=True, exist_ok=True)

        # Create or update justfile if it doesn't exist or is outdated
        if not justfile_path.exists():
            create_viewset_justfile(viewset_dir, viewset_name)


def create_viewset_justfile(viewset_dir: Path, viewset_name: str) -> None:
    """Create a justfile in the viewset directory for view management commands"""
    justfile_path = viewset_dir / "justfile"

    # Get the path to the main viewyard directory to reference scripts
    viewyard_dir = Path(__file__).parent.parent.resolve()

    justfile_content = f'''# Viewyard View Management for {viewset_name} viewset
# Auto-generated justfile for viewset directory

# Show available view commands
default:
    @just --list

# View management commands (auto-detects viewset from current directory)
view *args:
    python3 {viewyard_dir}/scripts/view-manager.py {{{{args}}}}

# List all views in this viewset
list:
    python3 {viewyard_dir}/scripts/view-manager.py list

# Create a new view in this viewset (no --viewset needed)
create view-name:
    python3 {viewyard_dir}/scripts/view-manager.py create {{{{view-name}}}}

# Delete a view from this viewset
delete view-name:
    python3 {viewyard_dir}/scripts/view-manager.py delete {{{{view-name}}}}

# Show information about a view
info view-name:
    python3 {viewyard_dir}/scripts/view-manager.py info {{{{view-name}}}}

# Add a repository to an existing view
add-repo view-name repo-name:
    python3 {viewyard_dir}/scripts/view-manager.py add-repo {{{{view-name}}}} {{{{repo-name}}}}

# Validate viewyard setup
validate:
    python3 {viewyard_dir}/scripts/view-manager.py validate
'''

    with open(justfile_path, 'w') as f:
        f.write(justfile_content)

    print_colored(f"Created justfile: {justfile_path}", "green")


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
    print("  just view create [--viewset <name>] <view-name>  # Create a new task view")
    print("  just view list                                   # List all views")
    print("  just view delete <name>                          # Delete a view")
    print("  just view info <name>                            # Show view information")
    print("  just view add-repo <view> <repo>                 # Add a repo to existing view")
    print("  just view validate                               # Validate setup")
    print("  just view validate-comprehensive [--auto-fix]    # Comprehensive validation with optional auto-fix")
    print("  just view setup-justfiles                        # Create justfiles in viewset directories")
    print("")
    print_colored("Viewsets:", "blue")
    print("  --viewset <name>            # Use specific viewset (defaults to first)")
    print("  Configuration: ~/.config/viewyard/viewsets.yaml")
    print("")
    print_colored("Working in a view:", "blue")
    print("  cd ~/src/<viewset>/views/<view-name>")
    print("  just status                 # Show status of repos in view")
    print("  just rebase                 # Rebase repos against origin/master")
    print("  just build                  # Build repos with changes")
    print("  just commit-all \"message\"   # Commit to all dirty repos")
    print("  just push-all               # Push repos with commits ahead")


def list_views() -> None:
    """List all views across all viewsets"""
    config = load_viewsets_config()
    viewsets = config.get("viewsets", {})

    if not viewsets:
        print_colored("No viewsets configured. Check ~/.config/viewyard/viewsets.yaml", "yellow")
        return

    print_colored("Available Views:", "blue")
    print("===============")
    print("")

    total_views = 0
    for viewset_name in viewsets.keys():
        views_dir = Path.home() / "src" / viewset_name / "views"

        if not views_dir.exists():
            continue

        views = [d.name for d in views_dir.iterdir() if d.is_dir()]

        if views:
            print_colored(f"Viewset: {viewset_name}", "blue")
            for view in sorted(views):
                view_path = views_dir / view
                repos_file = view_path / ".view-repos"

                if repos_file.exists():
                    with open(repos_file) as f:
                        repos = [line.strip() for line in f if line.strip()]
                    print(f"  {view} ({len(repos)} repos)")
                else:
                    print(f"  {view} (no repos configured)")
                total_views += 1
            print("")

    if total_views == 0:
        print_colored("No views found. Create your first view with: just view create <name>", "yellow")


def create_view(view_name: str, viewset_name: Optional[str] = None) -> None:
    """Create a new task view"""
    if not view_name:
        print_colored("Error: View name is required", "red")
        sys.exit(1)

    # Determine viewset name (use auto-detected, specified, or default)
    if not viewset_name:
        detected_viewset = detect_current_viewset()
        if detected_viewset:
            viewset_name = detected_viewset
            print_colored(f"Auto-detected viewset: {viewset_name}", "blue")
        else:
            config = load_viewsets_config()
            viewset_name = next(iter(config["viewsets"]))
            print_colored(f"Using default viewset: {viewset_name}", "blue")

    # Create viewset-specific views directory using absolute path
    viewset_dir = Path.home() / "src" / viewset_name
    views_dir = viewset_dir / "views"
    view_path = views_dir / view_name

    if view_path.exists():
        print_colored(f"Error: View '{view_name}' already exists in viewset '{viewset_name}'", "red")
        sys.exit(1)

    # Validate viewset directory if it exists
    if viewset_dir.exists():
        # Check if it looks like a valid viewset directory
        contents = list(viewset_dir.iterdir())
        non_views_items = [item for item in contents if item.name != "views"]

        if non_views_items:
            git_repos = [item for item in non_views_items if item.is_dir() and (item / ".git").exists()]
            if git_repos:
                print_colored(f"Warning: {viewset_dir} contains repositories outside of views/:", "yellow")
                for repo in git_repos:
                    print(f"  • {repo.name}/")
                print("These won't be managed by this view.")
                print("")

    # Create views directory if it doesn't exist
    views_dir.mkdir(parents=True, exist_ok=True)

    # Ensure viewset has a justfile
    justfile_path = viewset_dir / "justfile"
    if not justfile_path.exists():
        create_viewset_justfile(viewset_dir, viewset_name)

    print_colored(f"Creating view: {view_name} (viewset: {viewset_name})", "blue")
    print("")

    # Get available repos
    available_repos = get_available_repos(viewset_name)
    
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

        workspace_repos = load_workspace(viewset_name)

        for repo in selected_repos:
            print_colored(f"[{repo}]", "blue")

            # Find the repo config from viewset
            repo_config = next((r for r in workspace_repos if r["name"] == repo), None)

            if not repo_config:
                print_colored(f"  Error: Repository {repo} not found in viewset '{viewset_name}'", "red")
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


class ValidationResult:
    """Represents the result of a validation check"""
    def __init__(self, check_name: str, passed: bool, message: str, fix_function=None, fix_description: str = ""):
        self.check_name = check_name
        self.passed = passed
        self.message = message
        self.fix_function = fix_function
        self.fix_description = fix_description


class ViewyardValidator:
    """Comprehensive validation system for Viewyard setup"""

    def __init__(self, auto_fix: bool = False):
        self.auto_fix = auto_fix
        self.results = []

    def add_result(self, result: ValidationResult):
        """Add a validation result"""
        self.results.append(result)

    def validate_config_file(self) -> ValidationResult:
        """Validate viewsets configuration file exists and is valid"""
        config_file = Path.home() / ".config" / "viewyard" / "viewsets.yaml"

        if not config_file.exists():
            return ValidationResult(
                "Config File",
                False,
                f"❌ Missing: {config_file}",
                lambda: self._create_basic_config(config_file),
                "Create basic viewsets.yaml configuration"
            )

        try:
            with open(config_file) as f:
                config = yaml.safe_load(f)

            if not config or "viewsets" not in config:
                return ValidationResult(
                    "Config Structure",
                    False,
                    "❌ Invalid viewsets config structure",
                    lambda: self._fix_config_structure(config_file),
                    "Fix configuration structure"
                )
            elif not config["viewsets"]:
                return ValidationResult(
                    "Config Content",
                    False,
                    "❌ No viewsets defined",
                    None,
                    "Add viewsets to configuration"
                )
            else:
                return ValidationResult(
                    "Config File",
                    True,
                    "✅ Viewsets config is valid"
                )

        except yaml.YAMLError as e:
            return ValidationResult(
                "Config Parsing",
                False,
                f"❌ YAML parsing error: {e}",
                lambda: self._fix_yaml_syntax(config_file),
                "Fix YAML syntax errors"
            )
        except Exception as e:
            return ValidationResult(
                "Config Access",
                False,
                f"❌ Config error: {e}",
                None,
                "Check file permissions and content"
            )

    def validate_viewset_directories(self) -> List[ValidationResult]:
        """Validate that viewset directories exist and have correct structure"""
        results = []

        try:
            config = load_viewsets_config()
            viewsets = config.get("viewsets", {})

            for viewset_name, viewset_data in viewsets.items():
                # Check viewset directory exists
                viewset_dir = Path.home() / "src" / viewset_name
                if not viewset_dir.exists():
                    results.append(ValidationResult(
                        f"Viewset Directory ({viewset_name})",
                        False,
                        f"❌ Missing directory: {viewset_dir}",
                        lambda vd=viewset_dir: self._create_viewset_directory(vd),
                        f"Create {viewset_name} directory"
                    ))
                else:
                    results.append(ValidationResult(
                        f"Viewset Directory ({viewset_name})",
                        True,
                        f"✅ Directory exists: {viewset_dir}"
                    ))

                # Check views subdirectory exists
                views_dir = viewset_dir / "views"
                if not views_dir.exists():
                    results.append(ValidationResult(
                        f"Views Directory ({viewset_name})",
                        False,
                        f"❌ Missing views directory: {views_dir}",
                        lambda vd=views_dir: self._create_views_directory(vd),
                        f"Create {viewset_name}/views directory"
                    ))
                else:
                    results.append(ValidationResult(
                        f"Views Directory ({viewset_name})",
                        True,
                        f"✅ Views directory exists: {views_dir}"
                    ))

                # Check repository configuration
                if "repos" not in viewset_data or not viewset_data["repos"]:
                    results.append(ValidationResult(
                        f"Viewset Repos ({viewset_name})",
                        False,
                        f"❌ Viewset '{viewset_name}' has no repos",
                        None,
                        f"Add repositories to {viewset_name} viewset"
                    ))
                else:
                    repo_count = len(viewset_data["repos"])
                    results.append(ValidationResult(
                        f"Viewset Repos ({viewset_name})",
                        True,
                        f"✅ Viewset '{viewset_name}': {repo_count} repos"
                    ))

        except Exception as e:
            results.append(ValidationResult(
                "Viewset Validation",
                False,
                f"❌ Error validating viewsets: {e}",
                None,
                "Check viewsets configuration"
            ))

        return results

    def validate_justfiles(self) -> List[ValidationResult]:
        """Validate that justfiles exist in viewset directories"""
        results = []

        try:
            config = load_viewsets_config()
            viewsets = config.get("viewsets", {})

            for viewset_name in viewsets.keys():
                viewset_dir = Path.home() / "src" / viewset_name
                justfile_path = viewset_dir / "justfile"

                if not justfile_path.exists():
                    results.append(ValidationResult(
                        f"Justfile ({viewset_name})",
                        False,
                        f"❌ Missing justfile: {justfile_path}",
                        lambda vd=viewset_dir, vn=viewset_name: create_viewset_justfile(vd, vn),
                        f"Create justfile for {viewset_name}"
                    ))
                else:
                    # Check if justfile has correct content
                    try:
                        content = justfile_path.read_text()
                        if f"# Viewyard View Management for {viewset_name} viewset" in content:
                            results.append(ValidationResult(
                                f"Justfile ({viewset_name})",
                                True,
                                f"✅ Justfile exists and is valid: {justfile_path}"
                            ))
                        else:
                            results.append(ValidationResult(
                                f"Justfile ({viewset_name})",
                                False,
                                f"❌ Justfile exists but appears outdated: {justfile_path}",
                                lambda vd=viewset_dir, vn=viewset_name: create_viewset_justfile(vd, vn),
                                f"Update justfile for {viewset_name}"
                            ))
                    except Exception as e:
                        results.append(ValidationResult(
                            f"Justfile ({viewset_name})",
                            False,
                            f"❌ Error reading justfile: {e}",
                            lambda vd=viewset_dir, vn=viewset_name: create_viewset_justfile(vd, vn),
                            f"Recreate justfile for {viewset_name}"
                        ))

        except Exception as e:
            results.append(ValidationResult(
                "Justfile Validation",
                False,
                f"❌ Error validating justfiles: {e}",
                None,
                "Check viewsets configuration"
            ))

        return results

    def validate_git_config(self) -> ValidationResult:
        """Validate git configuration"""
        gitconfig_path = Path.home() / ".gitconfig"

        if not gitconfig_path.exists():
            return ValidationResult(
                "Git Config",
                False,
                "❌ No .gitconfig found",
                lambda: self._create_basic_gitconfig(gitconfig_path),
                "Create basic .gitconfig"
            )

        try:
            with open(gitconfig_path) as f:
                gitconfig_content = f.read()

            if "includeIf" in gitconfig_content and "viewyard" in gitconfig_content.lower():
                return ValidationResult(
                    "Git Config",
                    True,
                    "✅ Git config has viewyard includes"
                )
            else:
                return ValidationResult(
                    "Git Config",
                    False,
                    "❌ Git config missing viewyard includes",
                    lambda: self._add_viewyard_includes(gitconfig_path),
                    "Add viewyard includes to git config"
                )
        except Exception as e:
            return ValidationResult(
                "Git Config",
                False,
                f"❌ Error reading git config: {e}",
                None,
                "Check .gitconfig file permissions"
            )

    def validate_dependencies(self) -> List[ValidationResult]:
        """Validate required dependencies"""
        results = []

        # Check PyYAML
        try:
            import yaml as yaml_module
            results.append(ValidationResult(
                "PyYAML",
                True,
                "✅ PyYAML is available"
            ))
        except ImportError:
            results.append(ValidationResult(
                "PyYAML",
                False,
                "❌ PyYAML not installed",
                lambda: self._install_pyyaml(),
                "Install PyYAML: pip install PyYAML"
            ))

        # Check Just
        import subprocess
        try:
            subprocess.run(["just", "--version"], capture_output=True, check=True)
            results.append(ValidationResult(
                "Just",
                True,
                "✅ Just command runner is available"
            ))
        except (subprocess.CalledProcessError, FileNotFoundError):
            results.append(ValidationResult(
                "Just",
                False,
                "❌ Just command runner not found",
                None,
                "Install Just: https://github.com/casey/just#installation"
            ))

        return results

    def validate_repository_access(self) -> List[ValidationResult]:
        """Validate repository accessibility (SSH keys, GitHub access)"""
        results = []

        try:
            config = load_viewsets_config()
            viewsets = config.get("viewsets", {})

            # Test SSH connectivity to GitHub
            import subprocess
            try:
                result = subprocess.run(
                    ["ssh", "-T", "git@github.com"],
                    capture_output=True,
                    text=True,
                    timeout=10
                )
                if "successfully authenticated" in result.stderr:
                    results.append(ValidationResult(
                        "GitHub SSH",
                        True,
                        "✅ GitHub SSH authentication working"
                    ))
                else:
                    results.append(ValidationResult(
                        "GitHub SSH",
                        False,
                        "❌ GitHub SSH authentication failed",
                        None,
                        "Set up SSH keys for GitHub: https://docs.github.com/en/authentication/connecting-to-github-with-ssh"
                    ))
            except (subprocess.TimeoutExpired, subprocess.CalledProcessError, FileNotFoundError):
                results.append(ValidationResult(
                    "GitHub SSH",
                    False,
                    "❌ Cannot test GitHub SSH (ssh command failed)",
                    None,
                    "Install SSH client and set up GitHub SSH keys"
                ))

            # Test repository accessibility for each viewset
            for viewset_name, viewset_data in viewsets.items():
                repos = viewset_data.get("repos", [])
                accessible_count = 0
                total_count = len(repos)

                for repo in repos[:3]:  # Test first 3 repos to avoid being too slow
                    repo_url = repo.get("url", "")
                    if repo_url:
                        try:
                            # Test if we can reach the repository
                            result = subprocess.run(
                                ["git", "ls-remote", "--heads", repo_url],
                                capture_output=True,
                                text=True,
                                timeout=15
                            )
                            if result.returncode == 0:
                                accessible_count += 1
                        except (subprocess.TimeoutExpired, subprocess.CalledProcessError):
                            pass  # Repository not accessible

                if total_count > 0:
                    if accessible_count == min(3, total_count):
                        results.append(ValidationResult(
                            f"Repository Access ({viewset_name})",
                            True,
                            f"✅ Repositories accessible in {viewset_name} viewset"
                        ))
                    elif accessible_count > 0:
                        results.append(ValidationResult(
                            f"Repository Access ({viewset_name})",
                            False,
                            f"⚠️  Some repositories inaccessible in {viewset_name} viewset ({accessible_count}/{min(3, total_count)} tested)",
                            None,
                            "Check SSH keys and repository permissions"
                        ))
                    else:
                        results.append(ValidationResult(
                            f"Repository Access ({viewset_name})",
                            False,
                            f"❌ No repositories accessible in {viewset_name} viewset",
                            None,
                            "Check SSH keys, repository URLs, and permissions"
                        ))

        except Exception as e:
            results.append(ValidationResult(
                "Repository Access",
                False,
                f"❌ Error testing repository access: {e}",
                None,
                "Check network connectivity and SSH setup"
            ))

        return results

    # Fix methods
    def _create_basic_config(self, config_file: Path):
        """Create a basic viewsets.yaml configuration"""
        config_file.parent.mkdir(parents=True, exist_ok=True)
        basic_config = {
            "viewsets": {
                "work": {
                    "repos": []
                }
            }
        }
        with open(config_file, 'w') as f:
            yaml.dump(basic_config, f, default_flow_style=False)
        print_colored(f"Created basic configuration: {config_file}", "green")

    def _fix_config_structure(self, config_file: Path):
        """Fix configuration structure"""
        try:
            with open(config_file) as f:
                config = yaml.safe_load(f) or {}

            if "viewsets" not in config:
                config["viewsets"] = {"work": {"repos": []}}

            with open(config_file, 'w') as f:
                yaml.dump(config, f, default_flow_style=False)
            print_colored(f"Fixed configuration structure: {config_file}", "green")
        except Exception as e:
            print_colored(f"Failed to fix config structure: {e}", "red")

    def _fix_yaml_syntax(self, config_file: Path):
        """Attempt to fix YAML syntax errors"""
        print_colored(f"YAML syntax errors require manual fixing: {config_file}", "yellow")

    def _create_viewset_directory(self, viewset_dir: Path):
        """Create viewset directory"""
        viewset_dir.mkdir(parents=True, exist_ok=True)
        print_colored(f"Created viewset directory: {viewset_dir}", "green")

    def _create_views_directory(self, views_dir: Path):
        """Create views subdirectory"""
        views_dir.mkdir(parents=True, exist_ok=True)
        print_colored(f"Created views directory: {views_dir}", "green")

    def _create_basic_gitconfig(self, gitconfig_path: Path):
        """Create basic .gitconfig"""
        basic_config = """[user]
    name = Your Name
    email = your.email@example.com
"""
        gitconfig_path.write_text(basic_config)
        print_colored(f"Created basic .gitconfig: {gitconfig_path}", "green")
        print_colored("Please update with your actual name and email", "yellow")

    def _add_viewyard_includes(self, gitconfig_path: Path):
        """Add viewyard includes to git config"""
        print_colored("Adding viewyard includes requires manual setup", "yellow")
        print("Run the onboarding script to set up git configuration properly")

    def _install_pyyaml(self):
        """Install PyYAML"""
        print_colored("Please install PyYAML: pip install PyYAML", "yellow")

    def run_comprehensive_validation(self, show_passed: bool = True) -> bool:
        """Run all validation checks"""
        print_colored("🔍 Running Comprehensive Viewyard Validation", "blue")
        print("=" * 60)
        print("")

        # Run all validation checks
        self.add_result(self.validate_config_file())
        self.results.extend(self.validate_viewset_directories())
        self.results.extend(self.validate_justfiles())
        self.add_result(self.validate_git_config())
        self.results.extend(self.validate_dependencies())
        self.results.extend(self.validate_repository_access())

        # Display results
        passed_count = 0
        failed_count = 0

        for result in self.results:
            if result.passed:
                passed_count += 1
                if show_passed:
                    print_colored(result.message, "green")
            else:
                failed_count += 1
                print_colored(result.message, "red")

        print("")
        print_colored(f"Validation Summary: {passed_count} passed, {failed_count} failed",
                     "green" if failed_count == 0 else "yellow")

        # Handle failures
        if failed_count > 0:
            print("")
            print_colored("Issues Found:", "red")

            fixable_issues = []
            manual_issues = []

            for result in self.results:
                if not result.passed:
                    if result.fix_function:
                        fixable_issues.append(result)
                    else:
                        manual_issues.append(result)

            if fixable_issues:
                print("")
                print_colored("Automatically Fixable Issues:", "yellow")
                for result in fixable_issues:
                    print(f"  • {result.fix_description}")

                if self.auto_fix:
                    print("")
                    print_colored("Applying automatic fixes...", "blue")
                    for result in fixable_issues:
                        try:
                            result.fix_function()
                        except Exception as e:
                            print_colored(f"Failed to fix {result.check_name}: {e}", "red")
                else:
                    print("")
                    fix_choice = input("Apply automatic fixes? [y/N]: ").strip().lower()
                    if fix_choice == 'y':
                        print_colored("Applying fixes...", "blue")
                        for result in fixable_issues:
                            try:
                                result.fix_function()
                            except Exception as e:
                                print_colored(f"Failed to fix {result.check_name}: {e}", "red")

            if manual_issues:
                print("")
                print_colored("Manual Action Required:", "yellow")
                for result in manual_issues:
                    print(f"  • {result.fix_description}")

        if failed_count == 0:
            print("")
            print_colored("🎉 All validation checks passed! Viewyard is ready to use.", "green")
            print("")
            print("Try creating your first view:")
            print_colored("  just view create my-first-task", "cyan")
            return True
        else:
            print("")
            print_colored("⚠️  Some issues need attention. Run validation again after fixing.", "yellow")
            return False


def validate_setup() -> None:
    """Legacy validate_setup function - now uses comprehensive validation"""
    validator = ViewyardValidator(auto_fix=False)
    validator.run_comprehensive_validation(show_passed=True)


def validate_setup_comprehensive(auto_fix: bool = False, show_passed: bool = True) -> bool:
    """Run comprehensive validation with optional auto-fix"""
    validator = ViewyardValidator(auto_fix=auto_fix)
    return validator.run_comprehensive_validation(show_passed=show_passed)


def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        show_help()
        return

    # Extract viewset from arguments
    viewset_name, remaining_args = get_viewset_from_args(sys.argv[1:])

    if len(remaining_args) < 1:
        show_help()
        return

    command = remaining_args[0]

    if command == "create":
        view_name = remaining_args[1] if len(remaining_args) > 1 else ""
        create_view(view_name, viewset_name)
    elif command == "list":
        list_views()
    elif command == "delete":
        view_name = remaining_args[1] if len(remaining_args) > 1 else ""
        delete_view(view_name)
    elif command == "info":
        view_name = remaining_args[1] if len(remaining_args) > 1 else ""
        show_view_info(view_name)
    elif command == "add-repo":
        view_name = remaining_args[1] if len(remaining_args) > 1 else ""
        repo_name = remaining_args[2] if len(remaining_args) > 2 else ""
        add_repo_to_view(view_name, repo_name)
    elif command == "validate":
        validate_setup()
    elif command == "validate-comprehensive":
        auto_fix = "--auto-fix" in remaining_args
        validate_setup_comprehensive(auto_fix=auto_fix)
    elif command == "setup-justfiles":
        ensure_viewset_justfiles()
        print_colored("✅ Viewset justfiles created/updated", "green")
    else:
        show_help()


if __name__ == "__main__":
    main()
