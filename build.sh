#!/bin/bash
# SafeHold Simple Build Script
# For cross-platform builds, use build-universal.sh

set -e

echo "🚀 Building SafeHold (local development)..."

# Build CLI version
echo "📦 Building CLI version..."
cargo build --release --features cli

# Build GUI version
echo "🎨 Building GUI version..."
cargo build --release --features gui

echo "✅ Build completed successfully!"
echo "📁 Binaries available in: target/release/"