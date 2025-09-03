#!/bin/bash

# Fix git configurations for all existing viewyard views
# This script finds all git repositories in viewyard views and configures
# local git user settings based on the repository's GitHub account

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to map GitHub organization/remote to actual user account
map_to_user_account() {
    local repo_path="$1"
    local remote_url

    # Get the remote URL
    if ! remote_url=$(git -C "$repo_path" remote get-url origin 2>/dev/null); then
        echo ""
        return 1
    fi

    local user_account=""

    # Map based on remote URL patterns:
    if [[ "$remote_url" =~ git@github\.com-dheater:dheater/ ]]; then
        # Personal repos: git@github.com-dheater:dheater/repo.git -> dheater
        user_account="dheater"
    elif [[ "$remote_url" =~ git@github\.com:imprivata-pas/ ]]; then
        # Work repos: git@github.com:imprivata-pas/repo.git -> daniel-heater-imprivata
        user_account="daniel-heater-imprivata"
    elif [[ "$remote_url" =~ git@github\.com:dheater/ ]]; then
        # Personal repos without SSH alias: git@github.com:dheater/repo.git -> dheater
        user_account="dheater"
    elif [[ "$remote_url" =~ https://github\.com/dheater/ ]]; then
        # Personal repos HTTPS: https://github.com/dheater/repo.git -> dheater
        user_account="dheater"
    elif [[ "$remote_url" =~ https://github\.com/imprivata-pas/ ]]; then
        # Work repos HTTPS: https://github.com/imprivata-pas/repo.git -> daniel-heater-imprivata
        user_account="daniel-heater-imprivata"
    fi

    echo "$user_account"
}

# Function to configure git for a repository
configure_repo_git() {
    local repo_path="$1"
    local repo_name
    repo_name=$(basename "$repo_path")
    
    print_info "Configuring $repo_name..."
    
    # Map remote URL to user account
    local account
    account=$(map_to_user_account "$repo_path")

    if [[ -z "$account" ]]; then
        print_warning "Could not map remote URL to user account for $repo_name"
        return 1
    fi

    print_info "  Mapped to user account: $account"
    
    # Configure local git settings
    local email="${account}@users.noreply.github.com"
    
    # Set user.name
    if git -C "$repo_path" config --local user.name "$account" 2>/dev/null; then
        print_success "  Set user.name = $account"
    else
        print_error "  Failed to set user.name"
        return 1
    fi
    
    # Set user.email
    if git -C "$repo_path" config --local user.email "$email" 2>/dev/null; then
        print_success "  Set user.email = $email"
    else
        print_error "  Failed to set user.email"
        return 1
    fi
    
    # Set signing key if it exists in global config
    local global_signing_key
    if global_signing_key=$(git config --global user.signingkey 2>/dev/null); then
        if git -C "$repo_path" config --local user.signingkey "$global_signing_key" 2>/dev/null; then
            print_success "  Set user.signingkey = $global_signing_key"
        else
            print_warning "  Failed to set user.signingkey"
        fi
    fi
    
    print_success "Configured $repo_name"
    return 0
}

# Main function
main() {
    print_info "Finding all git repositories in viewyard views under ~/src..."
    
    # Find all .git directories that are in viewyard views
    local git_repos=()
    while IFS= read -r -d '' git_dir; do
        local repo_path
        repo_path=$(dirname "$git_dir")
        
        # Skip if this is a nested .git directory
        if [[ "$git_dir" == *"/.git/"* ]]; then
            continue
        fi
        
        # Check if this repo is in a viewyard structure
        # Look for .viewyard-repos.json in parent directories
        local current_dir="$repo_path"
        local is_viewyard_repo=false
        
        while [[ "$current_dir" != "/" && "$current_dir" != "$HOME" ]]; do
            if [[ -f "$current_dir/.viewyard-repos.json" ]]; then
                is_viewyard_repo=true
                break
            fi
            current_dir=$(dirname "$current_dir")
        done
        
        if [[ "$is_viewyard_repo" == true ]]; then
            git_repos+=("$repo_path")
        fi
    done < <(find ~/src -name ".git" -type d -print0 2>/dev/null)
    
    if [[ ${#git_repos[@]} -eq 0 ]]; then
        print_warning "No viewyard git repositories found under ~/src"
        return 0
    fi
    
    print_info "Found ${#git_repos[@]} viewyard repositories"
    echo
    
    local success_count=0
    local error_count=0
    
    # Configure each repository
    for repo_path in "${git_repos[@]}"; do
        if configure_repo_git "$repo_path"; then
            ((success_count++))
        else
            ((error_count++))
        fi
        echo
    done
    
    # Summary
    echo "=================================="
    print_info "Configuration Summary:"
    print_success "Successfully configured: $success_count repositories"
    if [[ $error_count -gt 0 ]]; then
        print_error "Failed to configure: $error_count repositories"
    fi
    echo "=================================="
    
    if [[ $error_count -eq 0 ]]; then
        print_success "All viewyard repositories are now properly configured!"
    else
        print_warning "Some repositories had configuration issues. Check the output above."
    fi
}

# Run main function
main "$@"
