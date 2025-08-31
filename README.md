# Viewyard

The refreshingly unoptimized alternative to monorepos.

A clean, simple workspace for coordinated development across multiple repositories using **task-based views** and **viewsets**.

## 🚀 Quick Start

```bash
# Create a task view (interactive)
viewyard view create CLIENTS-469

# Navigate to your isolated workspace
cd src-<viewset>/views/CLIENTS-469

# Start working
viewyard status
```

## 🎯 Core Concepts

### Task-Based Views
Instead of complex Git workflows, this workspace uses **isolated task views**:

- **One view per task** - complete isolation, no interference
- **Interactive setup** - choose only the repos you need
- **Clean lifecycle** - create → work → delete when done
- **Team-friendly** - simple commands, clear instructions

### Viewsets
Organize your repositories by context (work, personal, client, etc.):

- **Separate contexts** - work repos vs personal repos
- **Clean git config** - different credentials per viewset
- **Curated repo lists** - only the repos you actually use
- **Flexible organization** - organize however makes sense for you

## 📋 Main Commands

### View Management
```bash
viewyard view create <task-name>                    # Interactive repo selection (default viewset)
viewyard view create --viewset <name> <task-name>  # Use specific viewset
viewyard view list                                  # Show all views across viewsets
viewyard view delete <task> --force                # Clean up completed work
```

### Viewset Configuration
```bash
# Configuration file: ~/.config/viewyard/viewsets.yaml
# Example:
viewsets:
  work:
    repos:
      - name: api-service
        url: git@github.com:company/api-service.git
  personal:
    repos:
      - name: my-project
        url: git@github.com:me/my-project.git
```

### Within a View (cd ~/src/<viewset>/views/<task>/)
```bash
viewyard status                     # Status of all repos in view
viewyard commit-all "message"       # Commit to all dirty repos
viewyard push-all                   # Push repos with commits ahead
```

## 🏗️ How It Works

### 1. Configure Your Viewsets
```bash
# Create ~/.config/viewyard/viewsets.yaml
viewsets:
  work:
    repos:
      - name: api-service
        url: git@github.com:company/api-service.git
  personal:
    repos:
      - name: my-project
        url: git@github.com:me/my-project.git
```

### 2. Create a Task View
```bash
just view create CLIENTS-469
# Uses default viewset, shows numbered list of available repos
# Select: 1 2 (for api-service + another-service)
# Creates isolated workspace at ~/src/work/views/CLIENTS-469/

# Or specify a viewset:
just view create --viewset personal MY-FEATURE
# Creates workspace at ~/src/personal/views/MY-FEATURE/
```

### 3. Work in Complete Isolation
```bash
cd ~/src/work/views/CLIENTS-469
# You now have:
# - api-service/             (on CLIENTS-469 branch)
# - another-service/         (on CLIENTS-469 branch)
```

### 4. Use Smart Commands
```bash
viewyard commit-all "Fix bug"   # Commits to all dirty repos
viewyard push-all              # Pushes only repos with commits ahead
```

### 5. Clean Up When Done
```bash
viewyard view delete CLIENTS-469 --force
# Removes entire view - no Git complexity
```

## 🎯 Why Task-Based Views?

### ✅ **Simple and clean**
- **Perfect isolation** - views can't interfere with each other
- **Single branch context** - impossible to be on wrong branch
- **Clear workspace** - only repos needed for the task

### ✅ **Team-Friendly**
- **Interactive selection** - no need to remember repo names
- **Simple commands** - `just view create task-name`
- **Clear instructions** - exactly where to go and what to do

### ✅ **Smart Operations**
- **Build only changed repos** - saves time and resources
- **Commit coordination** - same message across related changes
- **Push coordination** - only repos that need pushing

## 📁 Workspace Structure

```
viewyard/                        # Coordination workspace (Rust binary)
├── src/                         # Source code
├── templates/                   # View templates
└── docs/                        # Documentation

~/src/<viewset>/views/           # Task workspaces
├── CLIENTS-469/                 # Your task view
│   ├── librssconnect/           # Repo on task branch
│   └── universal-connection-manager/
└── security-audit/              # Another task view
    ├── audit/
    └── connect/
```

## 🔧 Repository Management

Viewyard supports any Git repositories. Configure your available repositories in the workspace setup.

Example repositories:
- **librssconnect** - Core connection management library
- **universal-connection-manager** - Cross-platform connection manager UI
- **audit** - Security auditing and compliance tools
- **connect** - Connection orchestration services
- **parent** - Parent/guardian access management
- **go-sdk** - Go SDK for integration
- **third-party-deps** - Third-party dependencies

## 🚀 Getting Started

### Prerequisites
- Git with SSH access to your repositories
- Rust toolchain (for building viewyard)

### Quality Tools
Viewyard uses standard Cargo quality tools:
- `cargo fmt` - Automatic code formatting
- `cargo clippy` - Linting and static analysis
- `cargo test` - Run tests

These are configured in `Cargo.toml` and run automatically during development.

### Git Configuration for Multiple Contexts
Configure git to use different credentials for different viewsets:

```bash
# ~/.gitconfig
[includeIf "gitdir:~/src/src-work/"]
    path = ~/.gitconfig-work
[includeIf "gitdir:~/src/src-personal/"]
    path = ~/.gitconfig-personal

# ~/.gitconfig-work
[user]
    name = "Your Work Name"
    email = "you@company.com"

# ~/.gitconfig-personal
[user]
    name = "Your Personal Name"
    email = "you@personal.com"
```

### Setup
```bash
# Clone and build viewyard
git clone https://github.com/your-org/viewyard.git ~/src/viewyard
cd ~/src/viewyard
cargo build --release

# Configure your viewsets
mkdir -p ~/.config/viewyard
cp templates/viewsets/starter.yaml ~/.config/viewyard/viewsets.yaml
# Edit ~/.config/viewyard/viewsets.yaml with your repositories

# Create your first task view
viewyard view create my-first-task

# Navigate and start working
cd ~/src/<viewset>/views/my-first-task
viewyard status
```

## 💡 Examples

### Work Feature Development
```bash
# Create view for work feature (uses default viewset)
just view create CLIENTS-469
# Select: 1 2 (api-service + auth-service)

cd src-work/views/CLIENTS-469
just status                     # Check current state
# Make your changes...
just build                      # Build changed repos
just commit-all "Implement new feature"
just push-all                   # Push to GitHub
```

### Personal Project
```bash
# Create view for personal project
just view create --viewset personal my-feature
# Select repos from personal viewset

cd src-personal/views/my-feature
just status                     # Check current state
# Make your changes...
just build                      # Build changed repos
just commit-all "Add cool feature"
just push-all                   # Push to GitHub
```

### Multi-Context Development
```bash
# Work on both work and personal projects
just view create --viewset work WORK-123
just view create --viewset personal SIDE-PROJECT

# Switch between contexts easily
cd src-work/views/WORK-123      # Work context
cd src-personal/views/SIDE-PROJECT  # Personal context
```

## 🛠️ Advanced Usage

### Custom Repo Selection
```bash
# Specific repos for focused work
just view create ui-redesign universal-connection-manager

# Full stack development
just view create full-stack librssconnect universal-connection-manager audit connect
```

### View Management
```bash
# See all active views
just view list

# Quick cleanup
just view delete old-task force

# Multiple views for parallel work
just view create feature-a librssconnect
just view create feature-b universal-connection-manager
```

## 🔍 Troubleshooting

### Setup Issues
```bash
# Validate your setup
just view validate

# Re-run onboarding if needed
python3 scripts/onboard.py

# Check viewsets config
cat ~/.config/viewyard/viewsets.yaml
```

### View Creation Issues
```bash
# If view already exists
just view delete task-name force
just view create task-name

# Check what views exist
just view list

# If viewset not found
just view create --viewset work task-name
```

### Repository Issues
```bash
# If repos aren't cloning
# Check SSH access to GitHub
ssh -T git@github.com

# Check if repo exists in viewset
just view validate

# If branches don't exist
# They'll be created automatically
```

### Git Configuration Issues
```bash
# Check git config is working
cd src-work/views/some-view
git config user.email  # Should show work email

cd src-personal/views/some-view
git config user.email  # Should show personal email
```

## 🤝 Team Adoption

### For New Team Members
1. **Simple start**: `just view create my-task`
2. **Interactive prompts** guide through repo selection
3. **Clear next steps** after view creation
4. **Familiar Git workflows** within each repo

### For Existing Workflows
- **Incremental adoption** - use views for new tasks
- **Existing repos unchanged** - no disruption to current work
- **Optional coordination** - use view commands when helpful

### For Complex Projects
- **Multiple views** for different aspects of large projects
- **Coordinated commits** across related repositories
- **Clean handoffs** between team members

---

**Simple. Isolated. Reliable. The refreshingly unoptimized alternative to monorepos.**
