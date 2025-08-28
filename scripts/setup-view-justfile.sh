#!/bin/bash
# Script to set up justfile symlinks for view directories

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VIEWS_DIR="$(dirname "$SCRIPT_DIR")/views"

echo "Setting up justfile symlinks for view directories..."

cd "$VIEWS_DIR"

# Create symlinks for all view directories
for dir in */; do
    if [ -d "$dir" ] && [ ! -L "${dir}justfile" ]; then
        echo "Creating symlink for $dir"
        ln -sf ../justfile "${dir}justfile"
    fi
done

echo "Done! All view directories now have justfile symlinks."
