#!/bin/bash

# Chronos Server Build Script

set -e

echo "Building Chronos server..."

# Build the server with all server features
echo "Compiling Rust server with full features..."
cargo build --release --features server

echo "Server build complete!"
echo ""
echo "To run the server:"
echo "1. Make sure PostgreSQL is running and configured in .env"
echo "2. Run database migrations: cargo run --bin chronos -- migrate"
echo "3. Start the server: ./target/release/chronos"
echo ""
echo "Or for development:"
echo "cargo run --features server"