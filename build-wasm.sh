#!/bin/bash

# Chronos WebAssembly Build Script

set -e

echo "Building Chronos WebAssembly frontend..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WebAssembly package
echo "Compiling Rust to WebAssembly..."
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web --out-dir pkg --release -- --no-default-features --features wasm

echo "WebAssembly build complete!"
echo ""
echo "To serve the frontend:"
echo "1. Start the Chronos backend server: cargo run"
echo "2. Serve the frontend: python3 -m http.server 8080"
echo "3. Open http://localhost:8080 in your browser"
echo ""
echo "Or use any static file server to serve index.html"