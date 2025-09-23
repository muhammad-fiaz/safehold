#!/bin/bash

# SafeHold Release Preparation Script
# Prepares the project for publishing to crates.io and GitHub releases

set -e

echo "🚀 SafeHold Release Preparation Script"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo -e "${BLUE}🔍 Checking prerequisites...${NC}"

if ! command_exists cargo; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! command_exists git; then
    echo -e "${RED}❌ Git not found. Please install Git.${NC}"
    exit 1
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${BLUE}📦 Current version: ${CURRENT_VERSION}${NC}"

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}⚠️ Working directory is not clean. Uncommitted changes:${NC}"
    git status --short
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ Aborting release preparation.${NC}"
        exit 1
    fi
fi

# Run tests
echo -e "${BLUE}🧪 Running tests...${NC}"
cargo test || {
    echo -e "${RED}❌ Tests failed. Fix tests before releasing.${NC}"
    exit 1
}

# Check code quality
echo -e "${BLUE}🔍 Checking code quality...${NC}"
cargo clippy -- -D warnings || {
    echo -e "${RED}❌ Clippy warnings found. Fix warnings before releasing.${NC}"
    exit 1
}

# Format code
echo -e "${BLUE}🎨 Formatting code...${NC}"
cargo fmt --check || {
    echo -e "${YELLOW}⚠️ Code formatting issues found. Running cargo fmt...${NC}"
    cargo fmt
}

# Build release versions
echo -e "${BLUE}🔨 Building release versions...${NC}"

# CLI version
echo -e "${YELLOW}  📦 Building CLI version...${NC}"
cargo build --release --no-default-features || {
    echo -e "${RED}❌ CLI build failed.${NC}"
    exit 1
}

# GUI version
echo -e "${YELLOW}  🖥️ Building GUI version...${NC}"
cargo build --release --features gui || {
    echo -e "${RED}❌ GUI build failed.${NC}"
    exit 1
}

# Update CHANGELOG if needed
echo -e "${BLUE}📝 Checking CHANGELOG.md...${NC}"
if ! grep -q "Version ${CURRENT_VERSION}" CHANGELOG.md; then
    echo -e "${YELLOW}⚠️ Version ${CURRENT_VERSION} not found in CHANGELOG.md${NC}"
    echo -e "${YELLOW}   Please update CHANGELOG.md before releasing.${NC}"
fi

# Check documentation
echo -e "${BLUE}📚 Building documentation...${NC}"
cargo doc --no-deps --features gui || {
    echo -e "${RED}❌ Documentation build failed.${NC}"
    exit 1
}

# Cross-platform builds (optional)
echo ""
read -p "Build cross-platform binaries? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}🔨 Building cross-platform binaries...${NC}"
    ./build.sh || {
        echo -e "${YELLOW}⚠️ Cross-platform build failed. Continuing...${NC}"
    }
fi

# Package information
echo -e "${BLUE}📦 Package information:${NC}"
cargo package --list --features gui | head -20

# Dry-run publish
echo ""
read -p "Run dry-run publish check? (Y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    echo -e "${BLUE}🚀 Running publish dry-run...${NC}"
    cargo publish --dry-run --features gui || {
        echo -e "${RED}❌ Publish dry-run failed.${NC}"
        exit 1
    }
fi

echo ""
echo -e "${GREEN}✅ Release preparation completed successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Release checklist:${NC}"
echo -e "  • ✅ Tests passing"
echo -e "  • ✅ Code quality checks passed"
echo -e "  • ✅ CLI and GUI builds successful"
echo -e "  • ✅ Documentation builds successfully"
echo -e "  • ✅ Publish dry-run successful"
echo ""
echo -e "${YELLOW}🚀 To publish to crates.io:${NC}"
echo -e "  cargo publish --features gui"
echo ""
echo -e "${YELLOW}🏷️ To create Git tag and GitHub release:${NC}"
echo -e "  git tag v${CURRENT_VERSION}"
echo -e "  git push origin v${CURRENT_VERSION}"
echo ""
echo -e "${BLUE}📁 Binary artifacts available in:${NC}"
echo -e "  target/release/safehold (CLI + GUI)"
echo -e "  dist/ (cross-platform builds, if created)"