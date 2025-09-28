# Examples & Workflows

## 💡 Basic Workflows

### Work Feature Development
```bash
# Create viewset for work project
viewyard viewset create work-project
# Interactive selection will show work repositories
# Choose the repositories you need for this feature

cd work-project
viewyard view create FEATURE-456
cd FEATURE-456

# Work on the feature across repositories
viewyard status                 # Check current state
# Edit files in any repository...
viewyard commit-all "Implement feature 456"
viewyard push-all
```

### Personal Project
```bash
# Create viewset for personal project
viewyard viewset create my-side-project
# Interactive selection will show personal repositories
# Choose the repositories you need

cd my-side-project
viewyard view create new-feature
cd new-feature

# Work on your personal project
viewyard status                 # Check current state
# Edit files in any repository...
viewyard commit-all "Add new feature"
viewyard push-all
```

### Multi-Context Development
```bash
# Work on multiple projects simultaneously
viewyard viewset create work-feature
viewyard viewset create personal-project
viewyard viewset create bug-fix

# Switch between contexts easily
cd work-feature
viewyard view create TICKET-123
cd TICKET-123
# Work on work feature...

cd ../../personal-project
viewyard view create cool-idea
cd cool-idea
# Work on personal project...

cd ../../bug-fix
viewyard view create hotfix-789
cd hotfix-789
# Work on bug fix...
```

### Account-Specific Repository Discovery
```bash
# Discover repositories from a specific GitHub account
viewyard viewset create work-project --account dheater-work
viewyard viewset create personal-project --account dheater
```

## 🤖 Automation

### Manual Configuration
You can bypass the interactive selection entirely by creating the `.viewyard-repos.json` file directly:

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

Advanced fields you can add per repository:

- `"account"`: pick a specific GitHub identity for git config inside that
  repository.
- `"directory_name"`: override the on-disk folder name used during cloning.
  This is useful when two repositories share the same base name or when you
  want clearer local naming, while still pushing/pulling from the original
  remote. Leave it out to keep the default of cloning into `name`.

### Automation & Scripting
```bash
# Generate configurations programmatically
echo '[{"name":"repo1","url":"git@github.com:user/repo1.git","is_private":false,"source":"GitHub (user)"}]' > .viewyard-repos.json

# Use in CI/CD or testing environments
./scripts/generate-viewset-config.sh > .viewyard-repos.json
viewyard view create test-environment
```

### Version Control Integration
```bash
# Include viewyard config in your project repository
cd existing-project
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

git add .viewyard-repos.json
git commit -m "Add viewyard workspace configuration"

# Team members can now create views
viewyard view create feature-branch
```

## 🔄 Advanced Workflows

### Release Coordination
```bash
# Create release preparation workspace
viewyard viewset create release-v2.0
cd release-v2.0
viewyard view create release-prep
cd release-prep

# Coordinate release across repositories
viewyard status                    # Check all repos are clean
# Update version numbers, changelogs, etc.
viewyard commit-all "Prepare v2.0 release"
viewyard push-all

# Create release branches
cd frontend && git checkout -b release/v2.0 && cd ..
cd backend && git checkout -b release/v2.0 && cd ..
```

### Hotfix Workflow
```bash
# Create hotfix workspace
viewyard viewset create hotfix-critical
cd hotfix-critical
viewyard view create hotfix-security
cd hotfix-security

# Apply hotfix across repositories
viewyard status                    # Ensure clean state
# Apply security fixes...
viewyard commit-all "Security hotfix"
viewyard push-all

# Cherry-pick to release branches as needed
```

### Feature Branch Coordination
```bash
# Create feature workspace
viewyard viewset create user-auth
cd user-auth
viewyard view create feature/oauth-integration
cd feature/oauth-integration

# Develop feature across multiple repos
# Frontend: Add OAuth UI components
# Backend: Add OAuth endpoints
# Shared: Update authentication models

viewyard status                    # Check progress
viewyard commit-all "Add OAuth integration"
viewyard rebase                    # Rebase against latest main
viewyard push-all
```

### Testing Environment Setup
```bash
# Create testing workspace
viewyard viewset create integration-tests
cd integration-tests
viewyard view create test-env
cd test-env

# Set up consistent testing environment
# All repos on same branch for integration testing
viewyard status                    # Verify branch consistency
# Run integration tests across repositories
```

## 🎯 Best Practices

### Repository Selection
- **Group logically**: Include repositories that change together
- **Consider dependencies**: Include both dependent and dependency repositories

### Branch Management
- **Use descriptive names**: `feature/user-auth`, `bugfix/memory-leak`, `release/v2.0`
- **Keep views focused**: One view per feature/bug/task
- **Clean up regularly**: Delete views when work is complete
- **Coordinate timing**: Create views when ready to start work

### Workflow Integration
- **Check status frequently**: Use `viewyard status` to stay synchronized
- **Commit atomically**: Use `viewyard commit-all` for coordinated changes
- **Push together**: Use `viewyard push-all` to maintain synchronization
- **Rebase regularly**: Use `viewyard rebase` to stay current with main branches

### Team Coordination
- **Share configurations**: Version control `.viewyard-repos.json` files
- **Document workflows**: Include viewyard commands in project documentation
- **Standardize naming**: Agree on view naming conventions
- **Coordinate timing**: Communicate when creating/deleting shared views
