# Viewyard

The refreshingly unoptimized alternative to monorepos.

A clean, simple workspace for coordinated development across multiple repositories using **task-based views**.

## 🚀 Quick Start

```bash
# Create a task view (interactive)
just view create CLIENTS-469

# Navigate to your isolated workspace
cd views/CLIENTS-469

# Start working
just status
just build
```

## 🎯 Core Concept: Task-Based Views

Instead of complex Git workflows, this workspace uses **isolated task views**:

- **One view per task** - complete isolation, no interference
- **Interactive setup** - choose only the repos you need
- **Clean lifecycle** - create → work → delete when done
- **Team-friendly** - simple commands, clear instructions

## 📋 Main Commands

### View Management
```bash
just view create <task-name>        # Interactive repo selection
just view create <task> repo1 repo2 # Specify repos directly
just view list                      # Show all views
just view delete <task> force       # Clean up completed work
```

### Within a View (cd views/<task>/)
```bash
just status                         # Status of all repos in view
just rebase                         # Rebase against origin/master
just build                          # Build only repos with changes
just commit-all "message"           # Commit to all dirty repos
just push-all                       # Push repos with commits ahead
```

## 🏗️ How It Works

### 1. Create a Task View
```bash
just view create CLIENTS-469
# Shows numbered list of available repos
# Select: 1 2 (for librssconnect + universal-connection-manager)
# Creates isolated workspace at views/CLIENTS-469/
```

### 2. Work in Complete Isolation
```bash
cd views/CLIENTS-469
# You now have:
# - librssconnect/           (on CLIENTS-469 branch)
# - universal-connection-manager/ (on CLIENTS-469 branch)
# - justfile                 (view-specific commands)
```

### 3. Use Smart Commands
```bash
just build                  # Only builds repos with uncommitted changes
just commit-all "Fix bug"   # Commits to all dirty repos
just push-all              # Pushes only repos with commits ahead
```

### 4. Clean Up When Done
```bash
just view delete CLIENTS-469 force
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
viewyard/                        # Coordination workspace
├── justfile                     # Main commands
├── scripts/                     # Automation (nushell)
├── templates/                   # View templates
└── docs/                        # Documentation

views/                           # Task workspaces
├── CLIENTS-469/                 # Your task view
│   ├── librssconnect/           # Repo on task branch
│   ├── universal-connection-manager/
│   └── justfile                 # View commands
└── security-audit/              # Another task view
    ├── audit/
    ├── connect/
    └── justfile
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
- Just (command runner): `brew install just`
- Python 3 (for automation scripts)

### Setup
```bash
# Clone the coordination workspace
git clone git@github.com:daniel-heater-imprivata/viewyard.git ~/src
cd ~/src

# Create your first task view
just view create my-first-task

# Navigate and start working
cd views/my-first-task
just status
```

## 💡 Examples

### Feature Development
```bash
# Create view for magnet links feature
just view create CLIENTS-469
# Select: 1 2 (librssconnect + universal-connection-manager)

cd views/CLIENTS-469
just status                     # Check current state
# Make your changes...
just build                      # Build changed repos
just commit-all "Implement magnet links"
just push-all                   # Push to GitHub
```

### Security Audit
```bash
# Create view for security work
just view create security-audit audit connect
# Directly specify repos

cd views/security-audit
just rebase                     # Update against latest master
# Perform audit...
just commit-all "Security fixes"
just push-all
```

### Multi-Repo Bug Fix
```bash
# Create view for cross-repo bug fix
just view create BUG-123
# Interactive selection of affected repos

cd views/BUG-123
# Fix bug across multiple repos...
just commit-all "Fix authentication bug"
just push-all                   # Coordinated push
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

### View Creation Issues
```bash
# If view already exists
just view delete task-name force
just view create task-name

# Check what views exist
just view list
```

### Repository Issues
```bash
# If repos aren't cloning
# Check SSH access to GitHub
ssh -T git@github.com

# If branches don't exist
# They'll be created automatically
```

### Command Issues
```bash
# If nushell commands fail
# Make sure you're in the right directory
cd views/your-task-name
just status
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
