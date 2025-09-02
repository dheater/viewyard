#!/bin/bash
set -e

# Viewyard v0.2.0 Installation Script
# The refreshingly unoptimized alternative to monorepos

echo "🚀 Installing Viewyard v0.2.0..."

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ This installer is currently only for macOS"
    echo "   Please build from source: cargo build --release"
    exit 1
fi

# Check if cargo is available for building from source
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "📦 Cloning viewyard repository..."
git clone https://github.com/dheater/viewyard.git
cd viewyard

echo "🔨 Building release binary..."
cargo build --release

echo "📋 Installing to /usr/local/bin..."
sudo cp target/release/viewyard /usr/local/bin/viewyard
sudo chmod +x /usr/local/bin/viewyard

# Cleanup
cd /
rm -rf "$TEMP_DIR"

echo "✅ Viewyard v0.2.0 installed successfully!"
echo ""
echo "🎯 Quick start:"
echo "   viewyard --help                    # Show all commands"
echo "   viewyard view validate             # Validate your config"
echo "   viewyard view create my-task       # Create a new view"
echo "   viewyard status                    # Show repository status"
echo ""
echo "📚 Documentation: https://github.com/dheater/viewyard"
echo ""
echo "🐛 Found a bug? The new testing approach in v0.2.0 already found 3 critical bugs!"
echo "   Please report issues at: https://github.com/dheater/viewyard/issues"
