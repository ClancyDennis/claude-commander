#!/bin/bash

# Tauri Development Server Launcher
# This script properly sources the Rust environment before starting the dev server

echo "🚀 Starting Tauri Development Server..."
echo ""

# Source cargo environment
if [ -f "$HOME/.cargo/env" ]; then
    echo "📦 Loading Rust/Cargo environment..."
    source "$HOME/.cargo/env"
else
    echo "❌ Error: Cargo environment not found at ~/.cargo/env"
    echo "   Please install Rust: https://rustup.rs/"
    exit 1
fi

# Verify cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo command not found after sourcing environment"
    exit 1
fi

echo "✅ Cargo environment loaded"
echo ""

# Start the dev server
echo "🔨 Starting development server..."
echo "   This will compile the Rust backend and start the Tauri app"
echo ""

npm run tauri dev
