# Viewyard

Monorepo experience, multi-repo reality.

A clean, simple workspace for coordinated development across multiple repositories using **task-based views** and **viewsets**.

## 🎯 Benefits for Individual Developers

### ✅ **Simplified Workflow**
- **One command setup**: `viewyard view create my-task`
- **Interactive guidance** through repo selection
- **Clear workspace structure** - know exactly where everything is
- **Familiar Git workflows** within each repo

### ✅ **Better Organization**
- **Task isolation** - no interference between different work
- **Context separation** - work vs personal projects
- **Clean lifecycle** - create when starting, delete when done
- **Coordinated operations** - status, commit, push across repos

### ✅ **Reduced Complexity**
- **No complex Git workflows** - simple branching per task
- **No monorepo overhead** - keep repos independent
- **No workspace pollution** - each task gets fresh environment
- **No branch confusion** - always on the right branch for the task

## ⚠️ Important Disclaimers

### Security & Operations
- **Git Credentials**: Viewyard uses your existing git credentials to clone and manage repositories
- **Filesystem Operations**: This tool creates directories, clones repositories, and manages git branches
- **AI-Assisted Development**: This tool was developed with significant AI assistance. You decide your level of comfort with that.

### Scope
- **Repository Coordination Only**: Viewyard manages git repository views and coordination
- **Does NOT Replace Build Systems**: You still use your existing build tools (npm, cargo, make, etc.)
- **Does NOT Replace Testing**: You still use your existing test frameworks and CI/CD pipelines

## 🚀 Getting Started

### Prerequisites
- Git with SSH access to your repositories
- Github commandline for improved experience

### Quick Setup Example
Let's say you have two repositories for a web project:

```bash
# 3. Configure your repositories
viewyard onboard
# Interactive setup with:
# • Repository selection prompts
# • Automatic repository discovery from GitHub/Git
# • Configuration file creation
# • Setup validation
# Note: You can manually edit ~/.config/viewyard/viewsets.yaml later if needed

# 4. Create your first task view
viewyard view create add-user-auth
# Interactive prompt: Select repos (1 2 for both frontend and backend)

# 5. Navigate to your workspace
cd ~/src/default/views/add-user-auth

# 6. You now have isolated copies on the 'add-user-auth' branch:
# - my-frontend/    (on add-user-auth branch)
# - my-backend/     (on add-user-auth branch)

# 7. Work normally with your existing tools
cd my-frontend
npm install && npm run dev    # Use your normal build tools

cd ../my-backend
cargo test                    # Use your normal test tools

# 8. Coordinate across repos when ready
cd ..  # Back to view root
viewyard status               # See status of both repos
viewyard commit-all "Add user authentication"
viewyard push-all            # Push both repos
```

### What Just Happened?
- **Viewyard created isolated workspace** - separate from your main development
- **Each repo is on a task-specific branch** - no branch confusion
- **You use your existing tools** - npm, cargo, make, whatever you normally use
- **Viewyard coordinates git operations** - status, commits, pushes across repos
- **Clean separation** - when done, delete the view and you're back to clean state

## 🎯 Core Concepts

### Task-Based Views
Instead of complex Git workflows, this workspace uses **isolated task views** - workspaces containing copies of selected repositories, all automatically placed on the same task-specific branch. Repositories are duplicated across multiple views, but you don't need to think about branches since each view puts you on the right branch for that task. This allows you to quickly switch between or compare different pieces of work without branch confusion.

- **One view per task** - complete isolation, no interference
- **Interactive setup** - choose only the repos you need
- **Clean lifecycle** - create → work → delete when done
- **Simple commands** - straightforward CLI interface

### Viewsets
Organize your repositories by context (work, personal, client, etc.) to create a dynamic, personal monorepo-like experience. Unlike a true monorepo, repositories remain independent, but viewsets give you coordinated operations across them (status, commit, push, rebase) as if they were a single project.

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
# To clean up: manually delete view directory when done
```

### Viewset Configuration
```bash
# Configuration file: ~/.config/viewyard/viewsets.yaml
# Example:
viewsets:
  work:
    repos:
      - name: web-frontend
        url: git@github.com:company/web-frontend.git
      - name: api-backend
        url: git@github.com:company/api-backend.git
  personal:
    repos:
      - name: my-cli-tool
        url: git@github.com:me/my-cli-tool.git
```

### Within a View (cd ~/src/<viewset>/views/<task>/)
```bash
viewyard status                     # Status of all repos in view
viewyard commit-all "message"       # Commit to all dirty repos
viewyard push-all                   # Push repos with commits ahead
viewyard rebase                     # Rebase all repos against origin/main
```

## 🏗️ How It Works

### 1. Configure Your Viewsets
```bash
# Create ~/.config/viewyard/viewsets.yaml
viewsets:
  work:
    repos:
      - name: web-frontend
        url: git@github.com:company/web-frontend.git
      - name: api-backend
        url: git@github.com:company/api-backend.git
  personal:
    repos:
      - name: my-cli-tool
        url: git@github.com:me/my-cli-tool.git
```

### 2. Create a Task View
```bash
viewyard view create FEATURE-123
# Uses default viewset, shows numbered list of available repos
# Select: 1 2 (for web-frontend + api-backend)
# Creates isolated workspace at ~/src/work/views/FEATURE-123/

# Or specify a viewset:
viewyard view create --viewset personal MY-FEATURE
# Creates workspace at ~/src/personal/views/MY-FEATURE/
```

### 3. Work in Complete Isolation
```bash
cd ~/src/work/views/FEATURE-123
# You now have:
# - web-frontend/            (on FEATURE-123 branch)
# - api-backend/             (on FEATURE-123 branch)
```

### 4. Use Smart Commands
```bash
viewyard status                 # Check status of all repos
viewyard commit-all "Fix bug"   # Commits to all dirty repos
viewyard push-all              # Pushes only repos with commits ahead
viewyard rebase                # Rebase all repos against origin/main
```

### 5. Clean Up When Done
```bash
# When finished with the task, simply delete the view directory
rm -rf ~/src/default/views/FEATURE-123
# No Git complexity - just remove the isolated workspace
```



## 📁 Workspace Structure

```
~/src/<viewset>/views/           # Task workspaces
├── FEATURE-123/                 # Your task view
│   ├── web-frontend/            # Repo on task branch
│   └── api-backend/             # Repo on task branch
└── BUGFIX-456/                  # Another task view
    ├── web-frontend/
    └── database-migrations/
```



## 📋 View Templates

Templates allow you to save common repository configurations for reuse across multiple views.

### Creating Templates
```bash
# Add template configuration to viewsets.yaml
viewsets:
  work:
    repos:
      - name: web-frontend
        url: git@github.com:company/web-frontend.git
      - name: api-backend
        url: git@github.com:company/api-backend.git
      - name: database-migrations
        url: git@github.com:company/database-migrations.git
    templates:
      fullstack:
        repos: [web-frontend, api-backend, database-migrations]
      frontend-only:
        repos: [web-frontend]
      backend-only:
        repos: [api-backend, database-migrations]
```

### Using Templates
```bash
# Create view using a template
viewyard view create --template fullstack my-feature
# Automatically selects web-frontend, api-backend, and database-migrations

# Still works with interactive selection
viewyard view create my-feature
# Shows numbered list, you pick what you need
```

### Benefits of Templates
- **Consistent setups** - same repo combinations for similar work
- **Faster creation** - no need to remember which repos you need
- **Team sharing** - standardize common configurations
- **Flexible** - can still override with interactive selection

## 🔧 Advanced Configuration

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

## 💡 Examples

### Work Feature Development
```bash
# Create view for work feature (uses default viewset)
viewyard view create FEATURE-456
# Select: 1 2 (web-frontend + api-backend)

cd src-work/views/FEATURE-456
viewyard status                 # Check current state
# Make your changes...
viewyard commit-all "Implement new feature"
viewyard push-all               # Push to GitHub
```

### Personal Project
```bash
# Create view for personal project
viewyard view create --viewset personal my-feature
# Select repos from personal viewset

cd src-personal/views/my-feature
viewyard status                 # Check current state
# Make your changes...
viewyard commit-all "Add cool feature"
viewyard push-all               # Push to GitHub
```

### Multi-Context Development
```bash
# Work on both work and personal projects
viewyard view create --viewset work WORK-123
viewyard view create --viewset personal SIDE-PROJECT

# Switch between contexts easily
cd src-work/views/WORK-123      # Work context
cd src-personal/views/SIDE-PROJECT  # Personal context
```

## 🛠️ Advanced Usage

### View Management
```bash
# See all active views
viewyard view list

# Multiple views for parallel work
viewyard view create feature-a
viewyard view create feature-b
```

## 🔍 Troubleshooting

### Setup Issues
```bash
# Validate your setup
viewyard view validate

# Check viewsets config
cat ~/.config/viewyard/viewsets.yaml

# Test basic functionality
viewyard --help
```

### Repository Issues
```bash
# If repos aren't cloning
# Check SSH access to GitHub
ssh -T git@github.com

# Check git configuration
git config --list

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

## 🛠️ Development & Contributing

### Quality Tools
Viewyard uses standard Cargo quality tools:
- `cargo fmt` - Automatic code formatting
- `cargo clippy` - Linting and static analysis
- `cargo test` - Run tests

These are configured in `Cargo.toml` and run automatically during development.

### Building from Source
```bash
# Clone and build
git clone https://github.com/your-org/viewyard.git
cd viewyard
cargo build --release

# Run tests
cargo test

# Format and lint
cargo fmt
cargo clippy
```

### Contributing
- **Issues**: Report bugs and request features via GitHub Issues
- **Pull Requests**: Contributions welcome, please follow existing code style
- **Testing**: Add tests for new functionality
- **Documentation**: Update README and code comments as needed

---

**Simple. Isolated. Reliable. Monorepo experience, multi-repo reality.**
