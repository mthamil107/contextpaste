#!/bin/bash
# Development environment setup for ContextPaste
set -e

echo "=== ContextPaste Development Setup ==="
echo ""

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "ERROR: Rust not found. Install from https://rustup.rs/"
    exit 1
fi
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "Rust: $RUST_VERSION"

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "ERROR: Node.js not found. Install from https://nodejs.org/"
    exit 1
fi
NODE_VERSION=$(node --version)
echo "Node.js: $NODE_VERSION"

# Check pnpm
if ! command -v pnpm &> /dev/null; then
    echo "Installing pnpm..."
    npm install -g pnpm
fi
PNPM_VERSION=$(pnpm --version)
echo "pnpm: $PNPM_VERSION"

# Check Tauri CLI
if ! command -v cargo-tauri &> /dev/null; then
    echo "Installing Tauri CLI..."
    cargo install tauri-cli
fi

echo ""
echo "=== Installing dependencies ==="
pnpm install

echo ""
echo "=== Checking Rust compilation ==="
cd src-tauri && cargo check && cd ..

echo ""
echo "=== Setup complete! ==="
echo ""
echo "Run 'cargo tauri dev' to start development"
echo "Run './scripts/download-model.sh' to download AI model (optional, ~80MB)"
