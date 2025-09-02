# Security & Privacy

## What Viewyard Does

**Viewyard uses your existing tools:**
- Uses your GitHub CLI authentication to discover repositories
- Uses your git credentials (SSH keys or HTTPS tokens) to clone repositories
- Creates directories and files in your current working directory
- Runs standard git commands (clone, checkout, branch, etc.)

**All operations are local:**
- No data is sent anywhere except to GitHub (via your GitHub CLI)
- No telemetry, analytics, or usage tracking
- Configuration files only contain repository URLs and metadata
- Everything stays on your machine

## What Viewyard Does NOT Do

- Store or transmit your credentials anywhere
- Access system files or make system-wide changes
- Install git hooks or modify git configuration
- Send data to third-party services
- Analyze repository contents (only manages git metadata)

## Honest Risk Assessment

**Low risk stuff:**
- Repository discovery (read-only GitHub API calls)
- Creating directories and config files
- Standard git operations you could do manually

**Medium risk stuff:**
- Downloads repository contents to your machine
- Creates and switches git branches
- Uses your existing git credentials

**How to be safe:**
- Try it on non-critical repositories first
- Make sure important work is committed and pushed
- Understand what each command does before running it
- You can always `rm -rf viewset-name` to clean up

## If Something Goes Wrong

**Found a bug or security issue?**
- Open a GitHub issue with details
- For sensitive security stuff, contact me directly first

**Something broke your repositories?**
- Use git to restore to a known good state
- Report what happened so I can fix it

## The Fine Print

This is open source software (MIT license) provided "as is" without warranty. You're responsible for how you use it. If you work at a big company, check with your IT folks first.

**AI Development Disclosure:** This tool was developed with significant AI assistance. Make your own judgment about whether that matters to you.
