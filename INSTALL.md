# Installation & Setup

## Prerequisites

- **Git** with SSH access to your repositories
- **GitHub CLI** (`gh`) for repository discovery and authentication

## Installation

```bash
# Install from source (Rust required)
cargo install --git https://github.com/dheater/viewyard.git

# Or download binary from releases
# https://github.com/dheater/viewyard/releases
```

## Authentication Setup

### GitHub CLI Authentication
```bash
# Authenticate with GitHub
gh auth login

# Follow the prompts to:
# 1. Choose GitHub.com
# 2. Choose HTTPS or SSH (SSH recommended)
# 3. Authenticate via web browser
# 4. Choose SSH for git operations
```

### SSH Key Setup (Recommended)
```bash
# Generate SSH key if you don't have one
ssh-keygen -t ed25519 -C "your_email@example.com"

# Add SSH key to ssh-agent
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519

# Add SSH key to GitHub account
gh ssh-key add ~/.ssh/id_ed25519.pub --title "Viewyard Development"

# Test SSH connection
ssh -T git@github.com
```

### Multiple Account Setup
```bash
# Authenticate with work account
gh auth login --hostname github.com --web

# Switch between accounts
gh auth switch --hostname github.com
gh auth status
```

## Verification

```bash
# Verify GitHub CLI is working
gh auth status
gh repo list --limit 5

# Verify Viewyard is installed
viewyard --version
viewyard --help
```

## 🔍 Troubleshooting

### GitHub CLI Issues

**Authentication Failed:**
```bash
# Re-authenticate
gh auth logout
gh auth login

# Check authentication status
gh auth status

# Refresh authentication token
gh auth refresh
```

**Rate Limit Exceeded:**
```bash
# Check rate limit status
gh api rate_limit

# Wait for rate limit reset (usually 1 hour)
# Or use personal access token for higher limits
gh auth login --with-token < token.txt
```

### SSH Issues

**Permission Denied (publickey):**
```bash
# Test SSH connection
ssh -T git@github.com

# Check SSH agent
ssh-add -l

# Add SSH key to agent
ssh-add ~/.ssh/id_ed25519

# Verify SSH key is added to GitHub
gh ssh-key list
```

**SSH Key Not Found:**
```bash
# Generate new SSH key
ssh-keygen -t ed25519 -C "your_email@example.com"

# Add to GitHub
gh ssh-key add ~/.ssh/id_ed25519.pub
```

### Network Issues

**Connection Timeout:**
```bash
# Check internet connection
ping github.com

# Try HTTPS instead of SSH
git config --global url."https://github.com/".insteadOf git@github.com:

# Check GitHub status
curl -s https://www.githubstatus.com/api/v2/status.json
```

**Corporate Firewall:**
```bash
# Configure proxy if needed
git config --global http.proxy http://proxy.company.com:8080
git config --global https.proxy https://proxy.company.com:8080

# Or use HTTPS with authentication
gh auth login --web --hostname github.com
```

### Viewyard Issues

**Command Not Found:**
```bash
# Check if Viewyard is in PATH
which viewyard

# If installed via cargo, ensure ~/.cargo/bin is in PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Repository Not Found:**
```bash
# Verify repository exists and you have access
gh repo view owner/repository

# Check repository URL in .viewyard-repos.json
cat .viewyard-repos.json

# Verify SSH access
ssh -T git@github.com
```

**Permission Denied:**
```bash
# Check repository permissions
gh repo view owner/repository --json permissions

# Verify you're authenticated as the correct user
gh auth status

# Check if repository is private and you have access
gh repo list --limit 100 | grep repository-name
```

## Support

If you encounter issues not covered here:

1. Check the [Examples & Workflows](EXAMPLES.md) for usage patterns
2. Review [Security & Privacy](SECURITY.md) for security considerations
3. See [Contributing](CONTRIBUTING.md) for development setup
4. Open an issue on GitHub with detailed error messages and steps to reproduce
