#!/bin/bash

# SafeHold Cross-Platform Build Script
# Builds binaries for Windows, macOS, and Linux

set -e

echo "🔨 SafeHold Cross-Platform Build Script"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create dist directory
mkdir -p dist

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Function to build for a target
build_target() {
    local target=$1
    local name=$2
    
    echo -e "${BLUE}🔨 Building for $target...${NC}"
    
    # Add target if not already installed
    rustup target add $target 2>/dev/null || true
    
    # Build CLI version
    echo -e "${YELLOW}  📦 Building CLI version...${NC}"
    cargo build --release --target $target --no-default-features
    
    # Build GUI version
    echo -e "${YELLOW}  🖥️ Building GUI version...${NC}"
    cargo build --release --target $target --features gui
    
    # Create target directory
    mkdir -p "dist/$name"
    
    # Copy binaries based on target OS
    if [[ $target == *"windows"* ]]; then
        cp "target/$target/release/safehold.exe" "dist/$name/safehold-cli.exe"
        cp "target/$target/release/safehold.exe" "dist/$name/safehold-gui.exe"
    else
        cp "target/$target/release/safehold" "dist/$name/safehold-cli"
        cp "target/$target/release/safehold" "dist/$name/safehold-gui"
    fi
    
    # Copy documentation
    cp README.md "dist/$name/"
    cp CHANGELOG.md "dist/$name/"
    [ -f LICENSE ] && cp LICENSE "dist/$name/"
    
    echo -e "${GREEN}✅ Built $name${NC}"
}

# Windows targets
echo -e "${BLUE}🖥️ Building Windows targets...${NC}"
build_target "x86_64-pc-windows-msvc" "windows-x64"
build_target "x86_64-pc-windows-gnu" "windows-x64-gnu"

# macOS targets  
echo -e "${BLUE}🍎 Building macOS targets...${NC}"
build_target "x86_64-apple-darwin" "macos-x64"
build_target "aarch64-apple-darwin" "macos-arm64"

# Linux targets
echo -e "${BLUE}🐧 Building Linux targets...${NC}"
build_target "x86_64-unknown-linux-gnu" "linux-x64"
build_target "aarch64-unknown-linux-gnu" "linux-arm64"
build_target "x86_64-unknown-linux-musl" "linux-x64-musl"

echo ""
echo -e "${GREEN}🎉 All builds completed successfully!${NC}"
echo -e "${BLUE}📦 Binaries are available in the 'dist' directory:${NC}"
ls -la dist/

echo ""
echo -e "${YELLOW}📝 Note: Some targets may require additional setup:${NC}"
echo -e "  • ${YELLOW}Windows${NC}: Install Visual Studio Build Tools or MinGW"
echo -e "  • ${YELLOW}macOS${NC}: macOS SDK (available on macOS only)"
echo -e "  • ${YELLOW}Linux ARM64${NC}: Cross-compilation tools (gcc-aarch64-linux-gnu)"
echo -e "  • ${YELLOW}Linux MUSL${NC}: musl-tools package"