#!/usr/bin/env python3
"""Viewyard onboarding script - get new team members productive quickly"""

import os
import sys
import shutil
import subprocess
from pathlib import Path
from typing import Dict, List, Any

# Try to import yaml, provide helpful error if missing
try:
    import yaml
except ImportError:
    print("❌ PyYAML is required but not installed.")
    print("")
    print("Install it with one of these commands:")
    print("  pip install PyYAML")
    print("  pip3 install PyYAML")
    print("  python -m pip install PyYAML")
    print("  python3 -m pip install PyYAML")
    print("")
    print("Then run the onboarding script again.")
    sys.exit(1)


def print_colored(text: str, color: str = "blue") -> None:
    """Print colored text to terminal"""
    colors = {
        "red": "\033[31m",
        "green": "\033[32m",
        "yellow": "\033[33m",
        "blue": "\033[34m",
        "purple": "\033[35m",
        "cyan": "\033[36m",
        "reset": "\033[0m"
    }
    print(f"{colors.get(color, '')}{text}{colors['reset']}")


def print_header(text: str) -> None:
    """Print a section header"""
    print("")
    print_colored("=" * 60, "blue")
    print_colored(f" {text}", "blue")
    print_colored("=" * 60, "blue")
    print("")


def run_command(cmd: List[str], cwd: str = None) -> bool:
    """Run a command and return success status"""
    try:
        result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
        return result.returncode == 0
    except Exception:
        return False


def check_prerequisites() -> bool:
    """Check if required tools are installed"""
    print_header("Checking Prerequisites")

    checks = [
        ("Git", ["git", "--version"]),
        ("Just", ["just", "--version"]),
        ("Python 3", ["python3", "--version"]),
    ]

    all_good = True
    for name, cmd in checks:
        if run_command(cmd):
            print_colored(f"✓ {name} is installed", "green")
        else:
            print_colored(f"✗ {name} is missing", "red")
            all_good = False

    # Check PyYAML
    try:
        import yaml
        print_colored("✓ PyYAML is installed", "green")
    except ImportError:
        print_colored("✗ PyYAML is missing", "red")
        print("  Install with: pip install PyYAML")
        all_good = False

    return all_good


def get_git_config(key: str) -> str:
    """Get a git config value"""
    try:
        result = subprocess.run(["git", "config", "--global", key],
                                capture_output=True, text=True)
        if result.returncode == 0:
            return result.stdout.strip()
    except Exception:
        pass
    return None


def read_existing_git_configs() -> Dict[str, Dict[str, str]]:
    """Read existing context-specific git configs"""
    configs = {}
    home = Path.home()

    try:
        # Check for existing context config files
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

                if name_match or email_match:
                    configs[context_name] = {
                        'name': name_match or '',
                        'email': email_match or ''
                    }
            except Exception:
                continue

    except (PermissionError, OSError):
        # Handle permission errors or other OS-level issues gracefully
        pass

    return configs


def get_user_info() -> Dict[str, Any]:
    """Get user information for git config"""
    print_header("User Information")

    print("Let's set up your git configuration for different contexts.")
    print("This sets up git commit author info (what appears in 'git log').")
    print("")

    # Get current git config
    current_name = get_git_config("user.name")
    current_email = get_git_config("user.email")

    # Read existing context-specific configs
    existing_configs = read_existing_git_configs()
    if existing_configs:
        print(
            f"Found existing git configs for: {', '.join(existing_configs.keys())}")
        print("I'll use your existing configuration.")
        print("")

        # Use existing contexts
        contexts = []
        for context_name, config in existing_configs.items():
            print(
                f"✓ {context_name}: {config.get('name', 'Unknown')} <{config.get('email', 'Unknown')}>")
            contexts.append({
                'name': context_name,
                'git_name': config.get('name', current_name or 'Your Name'),
                'git_email': config.get('email', 'your.email@example.com')
            })

        # Ask if they want to modify
        print("")
        modify = input(
            "Do you want to modify these contexts? [y/N]: ").strip().lower()
        if modify != 'y':
            return {'contexts': contexts}

        print("")
        print("You can modify existing contexts or add new ones.")
        print("")

    print("How many git contexts do you need?")
    print("1. Just one - I use the same git identity everywhere")
    print("2. Two contexts - work and personal")
    print("3. Multiple contexts - I'll set them up myself")
    choice = input("Choose (1, 2, or 3) [2]: ").strip() or "2"
    print("")

    contexts = []

    if choice == "1":
        # Single context
        print_colored("Git Configuration:", "blue")
        name = input(f"Your full name [{current_name or 'Your Name'}]: ").strip(
        ) or current_name or "Your Name"
        email = input(f"Your email [{current_email or 'your@email.com'}]: ").strip(
        ) or current_email or "your@email.com"

        contexts.append({
            'name': 'default',
            'git_name': name,
            'git_email': email
        })

    elif choice == "3":
        # Multiple custom contexts
        print("Let's set up your contexts. You can add as many as you need.")
        print("Examples: work, personal, client-name, freelance, etc.")
        print("")

        context_count = 1
        while True:
            print_colored(f"Context #{context_count}:", "blue")

            context_name = input(
                "Context name (or press Enter to finish): ").strip()
            if not context_name:
                break

            name = input(f"  Full name for {context_name} [{current_name or 'Your Name'}]: ").strip(
            ) or current_name or "Your Name"
            email = input(f"  Email for {context_name}: ").strip()
            if not email:
                print_colored(
                    "  Email is required. Skipping this context.", "yellow")
                continue

            contexts.append({
                'name': context_name,
                'git_name': name,
                'git_email': email
            })

            print_colored(f"  ✓ Added {context_name} context", "green")
            print("")
            context_count += 1

    else:
        # Default: work and personal contexts
        # Try to detect existing work/personal contexts by email patterns
        work_context = None
        personal_context = None

        for context_name, config in existing_configs.items():
            email = config.get('email', '').lower()
            if any(
                work_indicator in email for work_indicator in [
                    '@company.com',
                    '@work.',
                    'imprivata',
                    'daniel.heater']):
                work_context = (context_name, config)
            elif any(personal_indicator in email for personal_indicator in ['@gmail.com', '@pm.me', '@personal.', 'dheater']):
                personal_context = (context_name, config)

        # Work context
        print_colored("Work/Company Information:", "blue")
        if work_context:
            context_name, config = work_context
            print(f"Using existing '{context_name}' context")
            work_name_default = config.get('name', current_name or 'Your Name')
            work_email_default = config.get('email', 'your.work@company.com')
            work_context_name = context_name
        else:
            work_name_default = current_name or "Your Name"
            work_email_default = current_email or "your.work@company.com"
            work_context_name = "work"

        work_name = input(f"Your full name for work commits [{work_name_default}]: ").strip(
        ) or work_name_default
        work_email = input(
            f"Your work email [{work_email_default}]: ").strip() or work_email_default

        print("")
        # Personal context
        print_colored("Personal Information:", "blue")
        if personal_context:
            context_name, config = personal_context
            print(f"Using existing '{context_name}' context")
            personal_name_default = config.get(
                'name', current_name or 'Your Name')
            personal_email_default = config.get(
                'email', 'your.personal@email.com')
            personal_context_name = context_name
        else:
            personal_name_default = current_name or "Your Name"
            # Try to guess personal email from global config
            if current_email and any(
                domain in current_email.lower() for domain in [
                    'gmail.com',
                    'yahoo.com',
                    'hotmail.com',
                    'outlook.com',
                    'pm.me']):
                personal_email_default = current_email
            else:
                personal_email_default = "your.personal@email.com"
            personal_context_name = "personal"

        personal_name = input(f"Your full name for personal commits [{personal_name_default}]: ").strip(
        ) or personal_name_default
        personal_email = input(f"Your personal email [{personal_email_default}]: ").strip(
        ) or personal_email_default

        contexts.extend([{'name': work_context_name,
                          'git_name': work_name,
                          'git_email': work_email},
                         {'name': personal_context_name,
                          'git_name': personal_name,
                          'git_email': personal_email}])

    return {'contexts': contexts}


def create_git_config(user_info: Dict[str, Any]) -> None:
    """Create git configuration files"""
    print_header("Setting Up Git Configuration")

    contexts = user_info.get('contexts', [])
    if not contexts:
        print_colored("No contexts to configure", "yellow")
        return

    home = Path.home()
    gitconfig_path = home / ".gitconfig"

    # Handle single context case
    if len(contexts) == 1 and contexts[0]['name'] == 'default':
        # Just update global git config
        context = contexts[0]
        subprocess.run(["git",
                        "config",
                        "--global",
                        "user.name",
                        context['git_name']],
                       check=True)
        subprocess.run(["git",
                        "config",
                        "--global",
                        "user.email",
                        context['git_email']],
                       check=True)
        print_colored("✓ Updated global git configuration", "green")
        return

    # Multiple contexts - create separate config files
    gitconfig_content = ""
    if gitconfig_path.exists():
        with open(gitconfig_path) as f:
            gitconfig_content = f.read()

    includes_needed = []
    config_files_created = []

    for context in contexts:
        context_name = context['name']
        config_filename = f".gitconfig-{context_name}"
        config_path = home / config_filename

        # Create include directive using the actual viewset directory structure
        # Note: This will be updated after viewsets are created with actual
        # names
        include_pattern = f'gitdir:~/src/{context_name}/'
        if include_pattern not in gitconfig_content:
            includes_needed.append(f"""
[includeIf "gitdir:~/src/{context_name}/"]
    path = ~/{config_filename}""")

        # Create context-specific config file
        config_content = f"""[user]
    name = "{context['git_name']}"
    email = "{context['git_email']}"

# Uncomment and configure if you want commit signing
# [gpg]
#     format = ssh
# [commit]
#     gpgsign = true
"""
        with open(config_path, 'w') as f:
            f.write(config_content)

        config_files_created.append(f"~/{config_filename}")

    # Add includes to main gitconfig
    if includes_needed:
        with open(gitconfig_path, 'a') as f:
            f.write("\n# Viewyard context-specific configs")
            for include in includes_needed:
                f.write(include)
        print_colored("✓ Updated ~/.gitconfig with context includes", "green")
    else:
        print_colored("✓ Git config includes already present", "green")

    # Report created files
    for config_file in config_files_created:
        print_colored(f"✓ Created {config_file}", "green")


def create_viewsets_config() -> None:
    """Create initial viewsets configuration"""
    print_header("Setting Up Viewsets")

    config_dir = Path.home() / ".config" / "viewyard"
    config_file = config_dir / "viewsets.yaml"

    if config_file.exists():
        print_colored("✓ Viewsets config already exists", "green")
        return

    config_dir.mkdir(parents=True, exist_ok=True)

    # Copy starter template
    template_path = Path("templates/viewsets/starter.yaml")
    if template_path.exists():
        shutil.copy2(template_path, config_file)
        print_colored("✓ Created starter viewsets config", "green")
    else:
        # Fallback if template doesn't exist
        template_config = {
            "viewsets": {
                "work": {
                    "repos": [
                        {
                            "name": "api-service",
                            "url": "git@github.com:company/api-service.git",
                            "build": "make",
                            "test": "make test"
                        }
                    ]
                },
                "personal": {
                    "repos": [
                        {
                            "name": "my-project",
                            "url": "git@github.com:username/my-project.git",
                            "build": "npm run build",
                            "test": "npm test"
                        }
                    ]
                }
            }
        }

        with open(config_file, 'w') as f:
            yaml.dump(template_config, f, default_flow_style=False, indent=2)
        print_colored("✓ Created basic viewsets config", "green")

    print_colored(f"  Location: {config_file}", "yellow")
    print("")
    print("Next steps:")
    print("1. Edit this file to add your actual repositories")
    print("2. Replace the example repos with your real ones")
    print("3. Add as many viewsets as you need (work, personal, client, etc.)")


def get_available_github_accounts() -> List[str]:
    """Get list of available GitHub accounts from gh auth status"""
    try:
        result = subprocess.run(["gh", "auth", "status"],
                              capture_output=True, text=True, timeout=10)
        if result.returncode == 0:
            # Parse accounts from output like "✓ Logged in to github.com account username"
            accounts = []
            for line in result.stdout.split('\n'):
                if "✓ Logged in to github.com account" in line:
                    # Extract account name from "✓ Logged in to github.com account username (keyring)"
                    parts = line.split("account ")
                    if len(parts) > 1:
                        account = parts[1].split(" ")[0].strip()
                        if account:
                            accounts.append(account)
            return accounts
    except Exception:
        pass
    return []


def discover_repositories_from_account(account: str) -> List[Dict[str, str]]:
    """Discover repositories from a specific GitHub account"""
    repos = []

    try:
        # Switch to the specified account
        switch_result = subprocess.run(["gh", "auth", "switch", "--user", account],
                                     capture_output=True, text=True, timeout=10)
        if switch_result.returncode != 0:
            print(f"    Failed to switch to account {account}")
            return repos

        # Get user repositories
        user_result = subprocess.run([
            "gh", "repo", "list", "--limit", "100", "--json", "name,url,isPrivate"
        ], capture_output=True, text=True, timeout=30)

        if user_result.returncode == 0:
            import json
            user_repos = json.loads(user_result.stdout)
            for repo in user_repos:
                privacy = " [private]" if repo.get("isPrivate", False) else ""
                repos.append({
                    "name": repo["name"],
                    "url": repo["url"],
                    "source": f"GitHub ({account}){privacy}"
                })

        # Get organization repositories
        orgs_result = subprocess.run([
            "gh", "api", "user/orgs", "--jq", ".[].login"
        ], capture_output=True, text=True, timeout=15)

        if orgs_result.returncode == 0:
            orgs = [org.strip() for org in orgs_result.stdout.split('\n') if org.strip()]
            print(f"    Checking {len(orgs)} organizations...")

            for org in orgs:
                org_result = subprocess.run([
                    "gh", "repo", "list", org, "--limit", "200", "--json", "name,url,isPrivate"
                ], capture_output=True, text=True, timeout=30)

                if org_result.returncode == 0:
                    org_repos = json.loads(org_result.stdout)
                    print(f"      Found {len(org_repos)} repos in {org}")
                    for repo in org_repos:
                        privacy = " [private]" if repo.get("isPrivate", False) else ""
                        repos.append({
                            "name": repo["name"],
                            "url": repo["url"],
                            "source": f"GitHub ({account}){privacy}"
                        })

    except Exception as e:
        print(f"    Error discovering repos from {account}: {e}")

    return repos


def discover_repositories() -> List[Dict[str, str]]:
    """Discover user's repositories from various sources"""
    repos = []

    # Check local directories for git repos
    home = Path.home()
    common_dirs = [
        home / "src",
        home / "code",
        home / "projects",
        home / "dev",
        home / "workspace",
        home / "repos"
    ]

    print("🔍 Discovering your repositories...")
    print("  • Scanning local directories...")
    local_count = 0

    for base_dir in common_dirs:
        if base_dir.exists():
            for item in base_dir.iterdir():
                if item.is_dir() and (item / ".git").exists():
                    # Get remote URL
                    try:
                        result = subprocess.run(
                            ["git", "-C", str(item), "remote", "get-url", "origin"],
                            capture_output=True, text=True, timeout=5
                        )
                        if result.returncode == 0:
                            remote_url = result.stdout.strip()
                            repos.append({
                                "name": item.name,
                                "path": str(item),
                                "url": remote_url,
                                "source": f"local ({base_dir.name})"
                            })
                            local_count += 1
                    except Exception:
                        # If no remote, still include as local repo
                        repos.append({
                            "name": item.name,
                            "path": str(item),
                            "url": f"file://{item}",
                            "source": f"local ({base_dir.name})"
                        })
                        local_count += 1

    print(f"    Found {local_count} local repositories")

    # Try to get GitHub repos if gh CLI is available
    print("  • Checking GitHub repositories...")
    github_count = 0
    try:
        # First, check if gh CLI is installed
        gh_check = subprocess.run(["gh", "--version"],
                                  capture_output=True, text=True, timeout=5)

        if gh_check.returncode != 0:
            print("    GitHub CLI not installed")
            print("    Install with: brew install gh")
            print("    Falling back to git-based repository discovery...")
        else:
            # Get all available GitHub accounts
            available_accounts = get_available_github_accounts()

            if not available_accounts:
                print("    GitHub CLI not authenticated")
                retry = input(
                    "    Would you like to authenticate now? [y/N]: ").strip().lower()
                if retry == 'y':
                    print("    Run: gh auth login")
                    print("    Then re-run this onboarding script.")
                    sys.exit(0)
                else:
                    print("    Continuing with local repositories and git-based discovery...")
            else:
                # Discover repositories from all available accounts
                print(f"    Found {len(available_accounts)} authenticated account(s): {', '.join(available_accounts)}")

                original_account = None
                try:
                    # Remember current account to restore later
                    current_result = subprocess.run(["gh", "api", "user", "--jq", ".login"],
                                                   capture_output=True, text=True, timeout=5)
                    if current_result.returncode == 0:
                        original_account = current_result.stdout.strip()
                except:
                    pass

                # Discover from each account
                for account in available_accounts:
                    print(f"    Discovering repositories from {account}...")
                    account_repos = discover_repositories_from_account(account)
                    repos.extend(account_repos)
                    github_count += len(account_repos)

                # Restore original account
                if original_account and original_account in available_accounts:
                    try:
                        subprocess.run(["gh", "auth", "switch", "--user", original_account],
                                     capture_output=True, text=True, timeout=5)
                    except:
                        pass

    except Exception as e:
        print(f"    Error accessing GitHub: {e}")
        print("    Falling back to git-based repository discovery...")
        # Fallback: try git-based discovery if GitHub CLI failed
        git_repos = discover_git_repositories()
        repos.extend(git_repos)
        github_count = len(git_repos)

    print(f"    Found {github_count} GitHub repositories")
    print(f"  • Total: {len(repos)} repositories discovered")
    print("")

    return repos


def fuzzy_search_repos(
        repos: List[Dict[str, str]], query: str) -> List[Dict[str, str]]:
    """Simple fuzzy search for repositories"""
    if not query:
        return repos

    query = query.lower()
    matches = []

    for repo in repos:
        name = repo["name"].lower()

        # Simple matching: exact match, starts with, or contains
        if query == name:
            score = 100
        elif name.startswith(query):
            score = 50
        elif query in name:
            score = 25
        else:
            continue  # No match

        matches.append((score, repo))

    # Sort by score (descending) and return repos
    matches.sort(key=lambda x: x[0], reverse=True)
    return [repo for score, repo in matches]


def analyze_viewset_directory(viewset_dir: Path) -> Dict[str, Any]:
    """Analyze existing viewset directory and determine if it's safe to proceed"""

    if not viewset_dir.exists():
        return {"safe_to_proceed": True, "message": None, "prompt": None}

    # Get directory contents
    contents = list(viewset_dir.iterdir())

    # Scenario 1: Empty directory
    if not contents:
        return {
            "safe_to_proceed": True,
            "message": f"Using existing empty directory {viewset_dir}",
            "prompt": None
        }

    # Check for views/ subdirectory (Viewyard managed)
    views_dir = viewset_dir / "views"
    has_views_dir = views_dir.exists() and views_dir.is_dir()

    # Scenario 2: Has views/ directory (likely Viewyard managed)
    if has_views_dir:
        view_dirs = [d for d in views_dir.iterdir() if d.is_dir()]
        if view_dirs:
            # Check for uncommitted changes in views
            active_views = []
            clean_views = []

            for view_dir in view_dirs:
                has_changes = check_view_has_changes(view_dir)
                if has_changes:
                    active_views.append(view_dir.name)
                else:
                    clean_views.append(view_dir.name)

            if active_views:
                view_list = "\n".join([f"  • {v} (has uncommitted changes)" for v in active_views] +
                                      [f"  • {v} (clean)" for v in clean_views])
                return {
                    "safe_to_proceed": False,
                    "message": f"Found existing Viewyard views in {viewset_dir}:\n{view_list}",
                    "prompt": "Continue adding repositories to this viewset? [y/N]: "}
            else:
                return {
                    "safe_to_proceed": True,
                    "message": f"Found existing clean Viewyard views in {viewset_dir}",
                    "prompt": None}
        else:
            # Empty views directory
            return {
                "safe_to_proceed": True,
                "message": f"Using existing Viewyard directory {viewset_dir}",
                "prompt": None
            }

    # Check for git repositories
    git_repos = [d for d in contents if d.is_dir() and (d / ".git").exists()]
    other_items = [item for item in contents if item !=
                   views_dir and item not in git_repos]

    # Scenario 4: Contains git repositories
    if git_repos:
        repo_list = "\n".join(
            [f"  • {repo.name}/ (git repository)" for repo in git_repos])
        if other_items:
            other_list = "\n".join(
                [f"  • {item.name}" for item in other_items])
            repo_list += f"\n{other_list}"

        return {
            "safe_to_proceed": False,
            "message": f"Directory {viewset_dir} contains repositories:\n{repo_list}",
            "prompt": """Options:
1. Use a different viewset name [default]
2. Continue (Viewyard views will be in views/ subdirectory)
3. Import these repositories into the viewset
Choose (1, 2, or 3) [1]: """}

    # Scenario 3: Contains other files/folders
    if other_items:
        item_list = "\n".join([f"  • {item.name}" for item in other_items])
        return {
            "safe_to_proceed": False,
            "message": f"Directory {viewset_dir} already exists and contains:\n{item_list}\n\nThis doesn't look like a Viewyard directory.",
            "prompt": """Options:
1. Use a different name for your viewset [default]
2. Continue anyway (Viewyard will create views/ subdirectory)
Choose (1 or 2) [1]: """}

    # Fallback
    return {"safe_to_proceed": True, "message": None, "prompt": None}


def check_view_has_changes(view_dir: Path) -> bool:
    """Check if a view directory has uncommitted changes in any repository"""
    try:
        for repo_dir in view_dir.iterdir():
            if repo_dir.is_dir() and (repo_dir / ".git").exists():
                # Check git status
                result = subprocess.run(
                    ["git", "-C", str(repo_dir), "status", "--porcelain"],
                    capture_output=True, text=True, timeout=5
                )
                if result.returncode == 0 and result.stdout.strip():
                    return True  # Has uncommitted changes
        return False
    except Exception:
        return False  # Assume clean if we can't check


def discover_git_repositories() -> List[Dict[str, str]]:
    """Discover repositories using git commands as fallback"""
    repos = []

    try:
        # Look for git repositories in common locations
        home = Path.home()
        search_dirs = [
            home / "src",
            home / "code",
            home / "projects",
            home / "dev",
            home / "workspace"
        ]

        for search_dir in search_dirs:
            if not search_dir.exists():
                continue

            print(f"    Scanning {search_dir}...")

            # Look for git repos in subdirectories
            for item in search_dir.iterdir():
                if not item.is_dir():
                    continue

                git_dir = item / ".git"
                if git_dir.exists():
                    try:
                        # Get remote URL
                        result = subprocess.run(
                            ["git", "-C", str(item), "remote", "get-url", "origin"],
                            capture_output=True, text=True, timeout=5
                        )

                        if result.returncode == 0:
                            remote_url = result.stdout.strip()

                            # Determine source from URL
                            source = "git (local)"
                            if "github.com" in remote_url:
                                if ":" in remote_url:
                                    # Extract account from SSH URL like
                                    # git@github.com:user/repo.git
                                    account_part = remote_url.split(
                                        ":")[1].split("/")[0]
                                    source = f"git (GitHub {account_part})"
                                elif "/" in remote_url:
                                    # Extract from HTTPS URL
                                    account_part = remote_url.split("/")[-2]
                                    source = f"git (GitHub {account_part})"

                            repos.append({
                                "name": item.name,
                                "path": str(item),
                                "url": remote_url,
                                "source": source
                            })

                    except Exception:
                        # Include repo even without remote
                        repos.append({
                            "name": item.name,
                            "path": str(item),
                            "url": "",
                            "source": "git (local only)"
                        })

        if repos:
            print(f"    Found {len(repos)} repositories via git")
        else:
            print("    No git repositories found in common directories")

    except Exception as e:
        print(f"    Git-based discovery failed: {e}")

    return repos


def get_suggested_viewset_name(
        git_context: str, discovered_repos: List[Dict[str, str]]) -> str:
    """Get suggested viewset name based on GitHub account and context"""

    # Look for GitHub account names in the discovered repos
    github_accounts = set()
    for repo in discovered_repos:
        source = repo.get("source", "").lower()
        if "github" in source:
            # Extract account name from source like "GitHub
            # (daniel-heater-imprivata)"
            if "(" in source and ")" in source:
                account_part = source.split("(")[1].split(")")[0]
                # Remove organization indicators
                if not any(
                    org in account_part for org in [
                        "imprivata-",
                        "company-"]):
                    github_accounts.add(account_part)

    if git_context == 'work':
        # For work context, look for work-related GitHub accounts
        work_accounts = [
            acc for acc in github_accounts if any(
                indicator in acc for indicator in [
                    "daniel-heater-imprivata",
                    "work",
                    "company"])]
        if work_accounts:
            # Use the work account name, but clean it up
            account = work_accounts[0]
            if account == "daniel-heater-imprivata":
                return "daniel-heater-imprivata"  # Keep full name for clarity
            return account

        # Fallback: try to detect from organization repos
        org_names = set()
        for repo in discovered_repos:
            source = repo.get("source", "").lower()
            if "imprivata" in source:
                org_names.add("imprivata")
            elif "company" in source:
                org_names.add("company")

        if "imprivata" in org_names:
            return "imprivata"
        elif org_names:
            return list(org_names)[0]
        else:
            return "work"

    elif git_context == 'personal':
        # For personal context, look for personal GitHub accounts
        personal_accounts = [acc for acc in github_accounts if acc not in [
            "daniel-heater-imprivata"] and not any(org in acc for org in ["imprivata", "company"])]
        if personal_accounts:
            return personal_accounts[0]  # Use the personal account name
        else:
            return "personal"  # Fallback

    else:
        # For other contexts, use the context name
        return git_context


def update_git_config_for_viewset(viewset_name: str, git_context: str) -> None:
    """Update git config to use actual viewset directory name"""
    gitconfig_path = Path.home() / ".gitconfig"

    if not gitconfig_path.exists():
        return

    try:
        with open(gitconfig_path) as f:
            content = f.read()

        # Replace the generic context path with the actual viewset path
        old_pattern = f'gitdir:~/src/{git_context}/'
        new_pattern = f'gitdir:~/src/{viewset_name}/'

        if old_pattern in content and old_pattern != new_pattern:
            updated_content = content.replace(old_pattern, new_pattern)

            with open(gitconfig_path, 'w') as f:
                f.write(updated_content)

            print_colored(
                f"✓ Updated git config to use ~/src/{viewset_name}/ directory",
                "green")

    except Exception as e:
        print_colored(f"Warning: Could not update git config: {e}", "yellow")


def create_viewsets_interactively() -> None:
    """Interactive creation of user's viewsets"""
    print_header("Setting Up Your Viewsets")

    config_file = Path.home() / ".config" / "viewyard" / "viewsets.yaml"

    # Load existing config
    try:
        with open(config_file) as f:
            config = yaml.safe_load(f)
    except Exception as e:
        print_colored(f"Error reading config: {e}", "red")
        return

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

    # Check what git contexts were configured to determine viewsets to create
    # Look at existing git config files to determine contexts
    existing_configs = read_existing_git_configs()
    git_contexts = []

    if existing_configs:
        # Use the actual context names from existing configs
        git_contexts = list(existing_configs.keys())
    else:
        # Fallback: check .gitconfig for includeIf patterns
        gitconfig_path = Path.home() / ".gitconfig"
        if gitconfig_path.exists():
            with open(gitconfig_path) as f:
                gitconfig_content = f.read()
                # Look for any includeIf patterns and extract context names
                import re
                patterns = re.findall(r'includeIf "gitdir:~/src/([^/]+)/"', gitconfig_content)
                if patterns:
                    git_contexts = list(set(patterns))  # Remove duplicates
                else:
                    # Legacy patterns
                    if 'gitdir:~/src/work/' in gitconfig_content:
                        git_contexts.append('work')
                    if 'gitdir:~/src/personal/' in gitconfig_content:
                        git_contexts.append('personal')

    # Default to work if no contexts detected
    if not git_contexts:
        git_contexts = ['work']

    # Check for existing viewsets before setting up new ones
    config_file = Path.home() / ".config" / "viewyard" / "viewsets.yaml"
    existing_viewsets = {}

    if config_file.exists():
        try:
            with open(config_file) as f:
                config = yaml.safe_load(f)
                existing_viewsets = config.get("viewsets", {})
        except Exception:
            pass

    # Filter out contexts that already have viewsets configured
    contexts_to_setup = []
    contexts_already_configured = []

    for context in git_contexts:
        # Check if there's already a viewset for this context
        # Look for exact match or similar names (e.g., 'dheater' context might have 'dheater' viewset)
        context_viewset_exists = False

        for viewset_name in existing_viewsets.keys():
            if (viewset_name.lower() == context.lower() or
                context.lower() in viewset_name.lower() or
                viewset_name.lower() in context.lower()):
                context_viewset_exists = True
                contexts_already_configured.append((context, viewset_name))
                break

        if not context_viewset_exists:
            contexts_to_setup.append(context)

    # Show status of existing and new viewsets
    if contexts_already_configured:
        print("✓ Found existing viewsets:")
        for context, viewset_name in contexts_already_configured:
            repo_count = len(existing_viewsets[viewset_name].get("repos", []))
            print(f"  • {viewset_name} ({context} context): {repo_count} repositories configured")
        print("")

        # Ask if user wants to update existing viewsets with newly discovered repos
        if contexts_already_configured and not contexts_to_setup:
            update_existing = input("Would you like to discover and add new repositories to existing viewsets? [y/N]: ").strip().lower()
            if update_existing == 'y':
                print("")
                print("🔍 Discovering repositories to update existing viewsets...")
                discovered_repos = discover_repositories()

                # Offer to update each existing viewset
                for context, viewset_name in contexts_already_configured:
                    print(f"\n{'='*60}")
                    print_colored(f"Updating viewset '{viewset_name}' ({context} context):", "blue")

                    # Filter repos for this context
                    context_repos = filter_repos_by_context(discovered_repos, context)
                    existing_repo_names = {repo["name"] for repo in existing_viewsets[viewset_name].get("repos", [])}
                    new_repos = [repo for repo in context_repos if repo["name"] not in existing_repo_names]

                    if new_repos:
                        print(f"Found {len(new_repos)} new repositories for {viewset_name}:")
                        for repo in new_repos[:5]:  # Show first 5
                            print(f"  • {repo['name']} ({repo['source']})")
                        if len(new_repos) > 5:
                            print(f"  ... and {len(new_repos) - 5} more")

                        add_new = input(f"Add these {len(new_repos)} repositories to {viewset_name}? [y/N]: ").strip().lower()
                        if add_new == 'y':
                            # Add new repos to existing viewset
                            existing_viewsets[viewset_name]["repos"].extend([
                                {"name": repo["name"], "url": repo["url"]} for repo in new_repos
                            ])

                            # Save updated config
                            with open(config_file, 'w') as f:
                                yaml.dump({"viewsets": existing_viewsets}, f, default_flow_style=False)

                            print_colored(f"✓ Added {len(new_repos)} repositories to {viewset_name}", "green")
                    else:
                        print(f"No new repositories found for {viewset_name}")

                print("\n✓ Viewset updates complete!")
                return

    if contexts_to_setup:
        print(f"Setting up new viewsets for: {', '.join(contexts_to_setup)}")
        print("")

        # Discover repositories once (this also detects current GitHub account)
        discovered_repos = discover_repositories()
    else:
        print("All contexts already have viewsets configured!")
        print("")
        print("Your viewsets are ready to use:")
        for context, viewset_name in contexts_already_configured:
            print(f"  • just view create <task-name> --viewset {viewset_name}")
        print("")
        print("To add more repositories to existing viewsets, re-run this script and choose 'y' when prompted.")
        return

    # Create each viewset with user-chosen names
    for i, git_context in enumerate(contexts_to_setup):
        if i > 0:
            print("\n" + "=" * 60)

        print_colored(
            f"Setting up viewset for '{git_context}' context:",
            "blue")

        # Get smart default based on GitHub account and context
        suggested_name = get_suggested_viewset_name(
            git_context, discovered_repos)

        viewset_name = input(
            f"Enter name for your {git_context} viewset [{suggested_name}]: ").strip() or suggested_name

        create_single_viewset(
            viewset_name,
            git_context,
            discovered_repos,
            config_file)

        # Update git config to use actual viewset directory name
        update_git_config_for_viewset(viewset_name, git_context)


def filter_repos_by_context(
        repos: List[Dict[str, str]], git_context: str) -> List[Dict[str, str]]:
    """Filter repositories based on git context (work vs personal)"""

    # Determine if this is a work or personal context based on common patterns
    is_work_context = any(work_indicator in git_context.lower() for work_indicator in
                         ['work', 'company', 'imprivata', 'daniel-heater-imprivata'])

    filtered_repos = []

    for repo in repos:
        source = repo.get("source", "").lower()

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


def import_existing_repositories(viewset_dir: Path) -> List[Dict[str, str]]:
    """Import existing git repositories from a directory"""
    repos = []

    for item in viewset_dir.iterdir():
        if item.is_dir() and (item / ".git").exists():
            # This is a git repository
            repo_name = item.name

            # Try to get remote URL
            try:
                result = subprocess.run(
                    ["git", "-C", str(item), "remote", "get-url", "origin"],
                    capture_output=True, text=True, timeout=5
                )
                if result.returncode == 0:
                    repo_url = result.stdout.strip()
                else:
                    repo_url = f"file://{item}"
            except Exception:
                repo_url = f"file://{item}"

            repos.append({
                "name": repo_name,
                "url": repo_url
            })

    return repos


def create_single_viewset(viewset_name: str,
                          git_context: str,
                          discovered_repos: List[Dict[str,
                                                      str]],
                          config_file: Path) -> None:
    """Create a single viewset interactively"""

    # Load existing config
    try:
        with open(config_file) as f:
            config = yaml.safe_load(f)
    except Exception as e:
        print_colored(f"Error reading config: {e}", "red")
        return

    # Create viewset directory first and handle existing content
    viewset_dir = Path.home() / "src" / viewset_name
    views_dir = viewset_dir / "views"

    # Initialize repos list (may be populated from existing viewset)
    repos = []

    if viewset_dir.exists():
        directory_status = analyze_viewset_directory(viewset_dir)
        if directory_status["message"]:
            print_colored(
                directory_status["message"],
                "green" if directory_status["safe_to_proceed"] else "yellow")

        if not directory_status["safe_to_proceed"]:
            if "repositories" in directory_status["message"]:
                # Offer to import existing repositories
                choice = input(
                    "Would you like to import existing repositories into this viewset? [y/N]: ").strip().lower()
                if choice == 'y':
                    repos = import_existing_repositories(viewset_dir)
                    print_colored(
                        f"✓ Imported {len(repos)} existing repositories", "green")
                else:
                    print(
                        "Skipping this viewset for now. You can set it up manually later.")
                    return
            else:
                print("Skipping this viewset for now. You can set it up manually later.")
                return

    # Create directory structure
    try:
        views_dir.mkdir(parents=True, exist_ok=True)
        print_colored(f"✓ Created viewset directory: {viewset_dir}", "green")

        # Create justfile for this viewset
        create_viewset_justfile_during_onboarding(viewset_dir, viewset_name)

    except Exception as e:
        print_colored(f"Error creating directory {viewset_dir}: {e}", "red")
        return

    # Filter repositories by git context (not viewset name)
    context_repos = filter_repos_by_context(discovered_repos, git_context)

    print("")
    if context_repos:
        print(f"Found {len(context_repos)} repositories for '{git_context}' context! Let's add them to your '{viewset_name}' viewset.")
        if len(context_repos) != len(discovered_repos):
            print(
                f"(Filtered from {len(discovered_repos)} total repositories based on {git_context} context)")
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
    else:
        print_colored(
            f"No repositories found for '{git_context}' context.", "yellow")

        # Provide helpful guidance based on context and current authentication
        current_auth = "unknown"
        try:
            import subprocess
            result = subprocess.run(["gh", "api", "user", "--jq", ".login"],
                                  capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                current_auth = result.stdout.strip()
        except:
            pass

        if git_context.lower() in ['personal', 'dheater'] and current_auth == 'daniel-heater-imprivata':
            print("This is because you're authenticated with your work GitHub account.")
            print("To discover personal repositories:")
            print("  1. Run: gh auth switch --user dheater")
            print("  2. Re-run this onboarding script")
            print("  3. Or manually add personal repositories below")
            print("")
        elif git_context.lower() in ['work', 'imprivata'] and current_auth == 'dheater':
            print("This is because you're authenticated with your personal GitHub account.")
            print("To discover work repositories:")
            print("  1. Run: gh auth switch --user daniel-heater-imprivata")
            print("  2. Re-run this onboarding script")
            print("  3. Or manually add work repositories below")
            print("")

        print_colored(f"Let's add repositories manually to your '{viewset_name}' viewset.", "blue")
        print("You can:")
        print("• Type a repository name to search through all discovered repositories")
        print("• Type '*' to add all discovered repositories")
        print("• Type 'search' to return to search mode (if repositories were found)")
        print("• Type 'done' when finished")
        print("• Or enter a repository name and URL manually")
        print("")

    # Unified repository selection interface
    manual_mode = not context_repos

    while True:
        # Show current selection before each prompt
        if repos:
            print_colored(
                f"Currently selected repositories ({len(repos)}):",
                "cyan")
            for i, repo in enumerate(repos, 1):
                print(f"  {i}. {repo['name']}")
            print("")

        if manual_mode:
            # Manual entry mode
            print_colored("Manual Repository Entry:", "blue")
            repo_name = input(
                "Repository name (or press Enter to return to search): ").strip()
            if not repo_name:
                if context_repos:
                    manual_mode = False  # Return to search mode
                    print("")
                    continue
                else:
                    break  # No discovered repos, so exit

            # Handle special commands
            if repo_name == "*":
                # Add all discovered repositories (use all discovered, not just context-filtered)
                repos_to_add = context_repos if context_repos else discovered_repos
                if repos_to_add:
                    added_count = 0
                    for repo in repos_to_add:
                        if not any(r["name"] == repo["name"] for r in repos):
                            repos.append(repo)
                            added_count += 1
                    print_colored(f"✓ Added {added_count} repositories", "green")
                    print("")
                    continue
                else:
                    print_colored("No discovered repositories to add", "yellow")
                    continue
            elif repo_name.lower() == "done":
                break
            elif repo_name.lower() == "search":
                # Allow searching through all discovered repos, even if context is empty
                if discovered_repos:
                    manual_mode = False
                    print("")
                    continue
                else:
                    print_colored("No discovered repositories to search", "yellow")
                    continue
            else:
                # Try to search for the repository name in all discovered repos
                search_results = fuzzy_search_repos(discovered_repos, repo_name)
                if search_results:
                    print("")
                    print("Found repositories:")
                    for i, repo in enumerate(search_results, 1):
                        print(f"   {i}. {repo['name']} ({repo['source']})")
                        print(f"       URL:  {repo['url']}")
                    print("")

                    try:
                        choice = input("Select repository number (or press Enter to search again): ").strip()
                        if choice:
                            choice_num = int(choice)
                            if 1 <= choice_num <= len(search_results):
                                selected_repo = search_results[choice_num - 1]
                                if not any(r["name"] == selected_repo["name"] for r in repos):
                                    repos.append(selected_repo)
                                    print_colored(f"✓ Added {selected_repo['name']}", "green")
                                else:
                                    print_colored(f"Repository {selected_repo['name']} already added", "yellow")
                                print("")
                                continue
                    except (ValueError, IndexError):
                        print_colored("Invalid selection", "yellow")
                        continue

                    # If no selection made, continue to manual URL entry
                    print("")

            repo_url = input(f"Git URL for {repo_name}: ").strip()
            if not repo_url:
                print_colored("Skipping - URL is required", "yellow")
                continue

            # Check for duplicates
            if any(r["name"] == repo_name for r in repos):
                print_colored(
                    f"Repository '{repo_name}' already added", "yellow")
                continue

            repos.append({
                "name": repo_name,
                "url": repo_url
            })

            print_colored(f"✓ Added {repo_name}", "green")
            print("")

        else:
            # Search mode
            query = input(
                "Search repositories (or 'done'/'manual'/'*'): ").strip()

            if query.lower() == 'done':
                break
            elif query.lower() == 'manual':
                manual_mode = True
                print("")
                continue
            elif query == '*':
                # Add all available repositories
                added_count = 0
                for repo in context_repos:
                    if not any(r["name"] == repo["name"] for r in repos):
                        repos.append({
                            "name": repo["name"],
                            "url": repo["url"]
                        })
                        added_count += 1

                print_colored(
                    f"✓ Added all {added_count} available repositories", "green")
                print("")
                continue

            # Show matching repositories
            if query:
                matches = fuzzy_search_repos(context_repos, query)
            else:
                matches = context_repos[:10]  # Show first 10 if no query

            if not matches:
                print_colored(
                    "No repositories found matching your search.", "yellow")
                continue

            print("")
            print("Found repositories:")
            for i, repo in enumerate(matches[:10], 1):
                source_info = f" ({repo['source']})" if repo['source'] else ""
                already_added = "✓ " if any(
                    r["name"] == repo["name"] for r in repos) else "  "
                print(f"{already_added}{i:2}. {repo['name']}{source_info}")
                if repo.get('path'):
                    print(f"       Path: {repo['path']}")
                print(f"       URL:  {repo['url']}")

            print("")
            selection = input(
                "Select repository number (or press Enter to search again): ").strip()

            if selection.isdigit():
                idx = int(selection) - 1
                if 0 <= idx < len(matches):
                    selected_repo = matches[idx]

                    # Check for duplicates
                    if any(r["name"] == selected_repo["name"] for r in repos):
                        print_colored(
                            f"Repository '{selected_repo['name']}' already added", "yellow")
                    else:
                        repos.append({
                            "name": selected_repo["name"],
                            "url": selected_repo["url"]
                        })

                        print_colored(
                            f"✓ Added {selected_repo['name']}", "green")
                    print("")
                else:
                    print_colored("Invalid selection.", "yellow")
            print("")

    if not repos:
        print_colored(
            "No repositories added. Keeping starter template.",
            "yellow")
        return

    # Update config with new viewset
    if "viewsets" not in config:
        config["viewsets"] = {}

    config["viewsets"][viewset_name] = {"repos": repos}

    # Write updated config
    try:
        with open(config_file, 'w') as f:
            yaml.dump(config, f, default_flow_style=False, indent=2)

        print_colored(
            f"✓ Created viewset '{viewset_name}' with {len(repos)} repositories",
            "green")
        print("")
        print("Your viewset is ready! You can:")
        print(
            f"  • Create views: just view create --viewset {viewset_name} <task-name>")
        print(f"  • Edit config: ~/.config/viewyard/viewsets.yaml")
        print(f"  • Add more repos later by editing the config file")

    except Exception as e:
        print_colored(f"Error saving config: {e}", "red")


def test_setup() -> None:
    """Test the viewyard setup - legacy function, now calls comprehensive validation"""
    run_post_onboarding_validation()


def create_viewset_justfile_during_onboarding(viewset_dir: Path, viewset_name: str) -> None:
    """Create a justfile in the viewset directory during onboarding"""
    justfile_path = viewset_dir / "justfile"

    # Get the path to the main viewyard directory to reference scripts
    # During onboarding, we're running from the viewyard directory
    viewyard_dir = Path.cwd().resolve()

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
    python3 {viewyard_dir}/scripts/view-manager.py validate-comprehensive
'''

    try:
        with open(justfile_path, 'w') as f:
            f.write(justfile_content)
        print_colored(f"✓ Created justfile: {justfile_path}", "green")
    except Exception as e:
        print_colored(f"Warning: Could not create justfile: {e}", "yellow")


def run_post_onboarding_validation() -> bool:
    """Run comprehensive validation after onboarding"""
    print_header("Validating Complete Setup")

    # Import the validation function
    import sys
    import os
    sys.path.insert(0, os.path.join(os.path.dirname(__file__)))

    try:
        # Import from view-manager.py
        import importlib.util
        spec = importlib.util.spec_from_file_location("view_manager",
                                                     os.path.join(os.path.dirname(__file__), 'view-manager.py'))
        view_manager = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(view_manager)

        # Run comprehensive validation with auto-fix enabled
        success = view_manager.validate_setup_comprehensive(auto_fix=True, show_passed=False)

        if success:
            print("")
            print_colored("🎉 Setup validation complete! Everything looks good.", "green")
            print("")
            print_colored("What's been set up:", "blue")

            # Show what was created
            config_file = Path.home() / ".config" / "viewyard" / "viewsets.yaml"
            if config_file.exists():
                with open(config_file) as f:
                    config = yaml.safe_load(f)
                    viewsets = config.get("viewsets", {})

                for viewset_name in viewsets.keys():
                    viewset_dir = Path.home() / "src" / viewset_name
                    print(f"  ✓ Viewset directory: {viewset_dir}")
                    print(f"  ✓ Views directory: {viewset_dir}/views/")
                    print(f"  ✓ Justfile: {viewset_dir}/justfile")

            print("")
            print_colored("Next steps:", "blue")
            print("1. Create your first view from any viewset directory:")
            print_colored("   cd ~/src/<viewset-name>", "cyan")
            print_colored("   just create my-first-task", "cyan")
            print("2. Or use the traditional command:")
            print_colored("   just view create my-first-task", "cyan")
            print("3. List available commands:")
            print_colored("   just view", "cyan")

        else:
            print("")
            print_colored("⚠️  Setup validation found some issues.", "yellow")
            print("Please address the issues above and run validation again:")
            print_colored("  python3 scripts/view-manager.py validate-comprehensive", "cyan")

        return success

    except Exception as e:
        print_colored(f"✗ Error running validation: {e}", "red")
        print("")
        print_colored("Basic setup appears complete, but validation failed.", "yellow")
        print("You can still try using viewyard:")
        print_colored("  just view create my-first-task", "cyan")
        return False


def main():
    """Main onboarding flow"""
    print_colored("Welcome to Viewyard!", "purple")
    print("This script will help you get set up quickly.")

    # Check prerequisites
    if not check_prerequisites():
        print("")
        print_colored(
            "Please install missing prerequisites and run again.",
            "red")
        sys.exit(1)

    # Get user info
    user_info = get_user_info()

    # Set up git config
    create_git_config(user_info)

    # Set up viewsets
    create_viewsets_config()

    # Create viewsets interactively
    create_viewsets_interactively()

    # Test setup
    test_setup()

    print("")
    print_colored("Next steps:", "blue")
    print("1. Create your first view: just view create <task-name>")
    print("2. Add more repositories by editing ~/.config/viewyard/viewsets.yaml")
    print("3. Check the README for more examples and usage")


if __name__ == "__main__":
    main()
