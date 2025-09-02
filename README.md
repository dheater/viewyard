# Viewyard

Monorepo experience, multi-repo reality.

Viewyard is a multi-repository workspace management tool that creates synchronized development environments across multiple Git repositories. Instead of forcing everything into a monorepo, Viewyard lets you work with multiple repositories as if they were a single codebase.

## 🚀 Quick Start

```bash
# 1. Install and authenticate
gh auth login                           # One-time GitHub CLI setup

# 2. Create a viewset (project workspace)
viewyard viewset create my-project      # Interactive repository selection

# 3. Create a view (synchronized branch workspace)  
cd my-project
viewyard view create feature-123        # Creates branch across all repos

# 4. Work normally - all repos are synchronized on the same branch
cd feature-123
# Edit files in any repository...

# 5. Coordinate across repositories
viewyard status                         # Status of all repos
viewyard commit-all "Add feature"       # Commit to all dirty repos  
viewyard push-all                       # Push all repos with commits
```

## 📋 Core Commands

### Viewset Management (project-level)
```bash
viewyard viewset create <name>          # Create new project workspace
viewyard viewset create <name> --account <github-user>  # From specific account
```

### View Management (branch-level)
```bash
viewyard view create <branch-name>      # Create synchronized branch workspace
```

### Workspace Commands (run from within a view directory)
```bash
viewyard status                         # Status of all repos (validates branch sync)
viewyard commit-all "message"           # Commit to all dirty repos
viewyard push-all                       # Push repos with commits ahead
viewyard rebase                         # Rebase all repos against their default branch
```

## 🏗️ How It Works

### 1. Authenticate with GitHub
```bash
# One-time setup - authenticate with GitHub CLI
gh auth login
# Supports multiple accounts for work/personal separation
```

### 2. Create a Viewset (Project Workspace)
```bash
viewyard viewset create my-project
# Interactive selection from your GitHub repositories
# Creates .viewyard-repos.json configuration file
```

### 3. Create Views (Synchronized Branch Workspaces)
```bash
cd my-project
viewyard view create feature-123
# Creates 'feature-123' branch in all repositories
# All repos are synchronized on the same branch
```

### 4. Work Across Repositories
```bash
cd feature-123
# Edit files in any repository
# All repositories are on the same branch for coordinated development
```

### 5. Coordinate Changes
```bash
viewyard status                 # See status of both repos (branch sync validated)
viewyard commit-all "Add user authentication"
viewyard push-all              # Push both repos
```

### 6. Create Additional Views
```bash
cd ..  # Back to viewset root
viewyard view create bug-fix-456  # Creates new synchronized branch workspace
```

## 🗂️ Directory Structure

```
my-project/                      # Viewset (project workspace)
├── .viewyard-repos.json        # Repository configuration
├── feature-123/                # View (synchronized branch workspace)
│   ├── web-frontend/           # Repository on 'feature-123' branch
│   └── api-backend/            # Repository on 'feature-123' branch
└── bug-fix-456/               # Another view
    ├── web-frontend/          # Repository on 'bug-fix-456' branch
    └── api-backend/           # Repository on 'bug-fix-456' branch
```

**Key Benefits:**
- **Minimal configuration** - only one JSON file per viewset (auto-generated)
- **Hierarchical organization** - viewsets contain multiple synchronized views
- **Branch synchronization** - all repos in a view are on the same branch
- **Automatic detection** - context determined from directory structure
- **Clean deletion** - just `rm -rf viewset-name` when done

## 🤖 Manual Configuration

You can bypass interactive selection by creating the `.viewyard-repos.json` file directly:

```bash
# Create a viewset directory
mkdir my-project && cd my-project

# Create the repository configuration manually
cat > .viewyard-repos.json << 'EOF'
[
  {
    "name": "frontend",
    "url": "git@github.com:myorg/frontend.git",
    "is_private": false,
    "source": "GitHub (myorg)"
  },
  {
    "name": "backend", 
    "url": "git@github.com:myorg/backend.git",
    "is_private": true,
    "source": "GitHub (myorg)"
  }
]
EOF

# Now create views normally
viewyard view create feature-123
```

## 📚 Documentation

- **[Installation & Setup](INSTALL.md)** - Prerequisites, GitHub CLI setup, troubleshooting
- **[Examples & Workflows](EXAMPLES.md)** - Detailed usage examples and team patterns
- **[Security & Privacy](SECURITY.md)** - Data handling and security considerations
- **[Contributing](CONTRIBUTING.md)** - Development setup and contribution guidelines

## 🔍 Repository Discovery

Viewyard automatically discovers repositories from your GitHub accounts:
- **Personal repositories** from your authenticated account
- **Organization repositories** from organizations you belong to
- **Multiple account support** for work/personal separation

## ⚡ Why Viewyard?

**Problems with monorepos:**
- Massive clone times and disk usage
- Complex build systems and tooling
- Tight coupling between unrelated code
- Difficult access control and team boundaries

**Problems with scattered repos:**
- No coordination between related changes
- Manual branch management across repositories
- Inconsistent development environments
- Complex release coordination

**Viewyard's solution:**
- **Synchronized branches** across multiple repositories
- **Lightweight coordination** without monorepo complexity
- **Flexible organization** - use multiple viewsets for different projects

---

Monorepo experience, multi-repo reality.
