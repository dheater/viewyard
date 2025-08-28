# Viewyard Development Tools
# The refreshingly unoptimized alternative to monorepos

# Repository list - keep in sync with setup.sh
repos := "librssconnect audit connect RDP2-Converter-rpm parent universal-connection-manager third-party-conan sl-conan-config go-sdk"

# Show available commands
default:
    @just --list









# === Task-Based Workspace Views ===

# View management - run 'just view' for help, or 'just view <subcommand>'
view *args:
    python3 scripts/view-manager.py {{args}}

# === View Coordination Commands (run from within a view directory) ===

# Show status of all repos in current view
status:
    python3 scripts/view-commands.py status

# Rebase all repos in current view against origin/master
rebase:
    python3 scripts/view-commands.py rebase

# Build repos with uncommitted changes in current view
build:
    python3 scripts/view-commands.py build

# Commit message to all dirty repos in current view
commit-all message:
    python3 scripts/view-commands.py commit-all "{{message}}"

# Push all repos with commits ahead in current view
push-all:
    python3 scripts/view-commands.py push-all
