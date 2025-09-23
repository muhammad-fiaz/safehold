# Contributing to SafeHold

Thank you for your interest in contributing to SafeHold! We welcome contributions from the community. This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project follows a code of conduct to ensure a welcoming environment for all contributors. By participating, you agree to:

- Be respectful and inclusive
- Focus on constructive feedback
- Accept responsibility for mistakes
- Show empathy towards other contributors
- Help create a positive community

## Getting Started

### Prerequisites

- **Rust**: Version 1.70 or higher ([install rustup](https://rustup.rs/))
- **Git**: For version control
- **Optional**: GUI dependencies for full development (eframe, egui_extras)

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/safehold.git
   cd safehold
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/muhammad-fiaz/safehold.git
   ```

## Development Setup

### Building the Project

SafeHold supports two build modes:

```bash
# Build CLI only (default - smaller binary, no GUI dependencies)
cargo build

# Build with GUI support (includes eframe/egui for graphical interface)
cargo build --features gui

# Build optimized release version
cargo build --release --features gui
```

**Note**: The GUI feature adds dependencies and increases binary size. Use `--features gui` only when you need the graphical interface.

### Running Tests

```bash
# Run all tests (includes unit and integration tests)
cargo test

# Run only integration tests (comprehensive CLI functionality tests)
cargo test --test integration_tests

# Run specific test module
cargo test test_name

# Run with verbose output
cargo test -- --nocapture

# Run tests for GUI features
cargo test --features gui
```

### Test Structure

- **Unit tests**: Located in `src/` modules, test individual functions
- **Integration tests**: Located in `tests/` directory:
  - `tests/integration_tests.rs`: Comprehensive CLI functionality tests
  - `tests/cli_basic.rs`: Basic CLI operation tests  
  - `tests/cli_export_run.rs`: Export and run functionality tests
  - `tests/cli_setup_launch.rs`: Setup and launch functionality tests

**Note**: Integration tests use isolated test environments with `SAFEHOLD_HOME` environment variable to prevent interference with your actual SafeHold data.

### Development Commands

```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open

# Cross-platform building (requires targets installed)
./build.sh          # Unix/Linux/macOS
build.bat           # Windows

# Build for specific target
cargo build --target x86_64-pc-windows-msvc --release
cargo build --target x86_64-apple-darwin --release --features gui
cargo build --target x86_64-unknown-linux-gnu --release
```

### Cross-Platform Development

SafeHold supports multiple platforms with dedicated build scripts:

- **Unix/Linux/macOS**: Use `./build.sh` for automated cross-compilation
- **Windows**: Use `build.bat` for Windows-based cross-compilation
- **GitHub Actions**: Automated builds for all platforms on release

The build system creates binaries for:
- Windows (x64 MSVC and GNU)
- macOS (Intel x64 and Apple Silicon ARM64)
- Linux (x64 GNU, ARM64, and MUSL static)

### Update Checking System

SafeHold includes an automatic update checking system:

- **Background checking**: Non-blocking update checks on every command
- **CLI command**: `safehold check-update` for manual checking
- **GUI integration**: Update notifications in GUI with modal dialogs
- **Version comparison**: Semantic version parsing and comparison
- **Internet connectivity**: Graceful handling of offline scenarios

When adding features that interact with updates:
- Use the `utils::update_checker` module
- Implement async functions with proper error handling
- Test both online and offline scenarios

## How to Contribute

### Types of Contributions

- **Bug fixes**: Fix existing issues
- **Features**: Add new functionality (e.g., CLI commands, GUI features)
- **Documentation**: Improve docs, README, examples, docstrings
- **Tests**: Add or improve test coverage (unit tests, integration tests)
- **Code quality**: Refactoring, performance improvements
- **Cross-platform**: Improve Windows, macOS, and Linux compatibility

### Feature Areas

SafeHold includes several key areas for contribution:

- **CLI Commands**: Adding new commands or improving existing ones
  - Project management (`create`, `list-projects`, `delete-project`)
  - Credential management (`add`, `get`, `update`, `delete`, `list`)
  - Global credentials (`global-add`, `global-get`, `global-update`, `global-delete`, `global-list`)
  - Statistics and counting (`count` with various options)
  - Export and execution (`export`, `run`, `show-all`)
  - Utilities (`setup`, `clean`, `launch`)

- **GUI Interface**: Enhancing the graphical interface
  - Main project view and credential management
  - Global credentials tab
  - Settings display (version, author information)
  - Dialog implementations for update/delete operations

- **Security & Encryption**: Improving secure storage
- **Cross-platform Support**: Ensuring functionality across all platforms

### Finding Issues

- Check the [Issues](https://github.com/muhammad-fiaz/safehold/issues) page
- Look for issues labeled `good first issue` or `help wanted`
- Comment on issues you'd like to work on to avoid duplicate work

## Development Workflow

1. **Choose an issue** or create one describing your planned changes
2. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-number-description
   ```
3. **Make your changes** following the guidelines below
4. **Test your changes** thoroughly
5. **Commit your changes** with clear, descriptive messages:
   ```bash
   git commit -m "feat: add new feature description"
   # or
   git commit -m "fix: resolve issue with detailed description"
   ```
6. **Push your branch**:
   ```bash
   git push origin your-branch-name
   ```
7. **Create a Pull Request** on GitHub

## Docker Development

SafeHold supports Docker for development and deployment. This section covers working with Docker during development.

### Docker Setup for Development

The project includes Docker configuration for containerized development and testing:

```bash
# Build the Docker image locally
docker build -t safehold:dev .

# Run SafeHold in a container
docker run --rm -it safehold:dev --help

# Use docker-compose for development (recommended)
docker-compose up --build
```

### Container Development Workflow

1. **Local development with Docker**:
   ```bash
   # Build and test in container
   docker build -t safehold:test .
   docker run --rm safehold:test cargo test
   
   # Interactive development container
   docker run --rm -it -v "$(pwd):/workspace" -w /workspace rust:1.75 bash
   ```

2. **Testing with containers**:
   ```bash
   # Run full test suite in container
   docker run --rm -v "$(pwd):/workspace" -w /workspace rust:1.75 cargo test
   
   # Test specific features
   docker run --rm -v "$(pwd):/workspace" -w /workspace rust:1.75 cargo test --features gui
   ```

3. **Cross-platform testing**:
   ```bash
   # Test on different architectures using Docker buildx
   docker buildx build --platform linux/amd64,linux/arm64 -t safehold:multi .
   ```

### Docker Configuration Files

- **Dockerfile**: Multi-stage build for optimized production images
  - Builder stage: Rust compilation with full toolchain
  - Runtime stage: Minimal Debian image with only necessary dependencies
  - Security: Non-root user execution

- **docker-compose.yml**: Development orchestration
  - Persistent data volumes for SafeHold data
  - Environment variable configuration
  - Health checks and restart policies

- **.dockerignore**: Excludes unnecessary files from build context
  - Target directory and build artifacts
  - Git metadata and documentation
  - Test files and temporary data

### Container Testing Guidelines

When developing with Docker:

1. **Test isolation**: Containers provide clean environments for testing
2. **Volume mapping**: Use volumes for persistent data during development
3. **Environment variables**: Test different configurations using environment variables
4. **Multi-stage builds**: Verify both build and runtime stages work correctly

### Container Security Considerations

- **Non-root execution**: Container runs as non-root user `safehold`
- **Minimal runtime**: Production image based on `debian:12-slim`
- **No secrets in images**: Never include credentials or sensitive data in Docker images
- **Health checks**: Container includes health check endpoint

### Docker Best Practices for Contributors

1. **Keep images small**: Use multi-stage builds and minimal base images
2. **Cache optimization**: Order Dockerfile commands for optimal layer caching
3. **Security scanning**: Regularly scan images for vulnerabilities
4. **Documentation**: Update Docker documentation when modifying container behavior

### Troubleshooting Docker Issues

Common Docker development issues:

```bash
# Clear Docker build cache
docker builder prune

# Remove all containers and rebuild
docker-compose down --volumes
docker-compose up --build

# Check container logs
docker-compose logs

# Debug container issues
docker run --rm -it --entrypoint /bin/bash safehold:dev
```

### Feature Development Guidelines

#### Adding New Commands

When adding new CLI commands to SafeHold:

1. **Add to CLI structure** in `src/cli/cli.rs`:
   - Add new command variant to `Commands` enum
   - Include descriptive help text and aliases
   - Add appropriate command line arguments

2. **Implement command logic** in `src/operations/envops.rs`:
   - Create new function following naming convention `cmd_<command_name>`
   - Add comprehensive docstring documentation
   - Include error handling and user feedback

3. **Update dispatch logic** in `src/cli/cli.rs`:
   - Add routing to your new function in the `dispatch` function

4. **Add GUI support** if applicable in `src/gui/ui.rs`:
   - Add UI elements and dialogs for the new functionality
   - Ensure consistency with existing GUI patterns

#### Destructive Operations Guidelines

⚠️ **CRITICAL**: When adding destructive operations:

1. **Always include confirmation prompts** unless `--force` flag is used
2. **Provide clear warnings** about what will be deleted/lost
3. **Use specific confirmation text** (like "DELETE ALL MY DATA") for dangerous operations
4. **Add comprehensive safety documentation** in help text and README
5. **Test thoroughly** in isolated environments only
6. **Add both force and interactive modes** for automation and safety

#### Command Naming Conventions

- Use clear, descriptive names (e.g., `clean-cache` not `clean`)
- Provide intuitive aliases (e.g., `clear-cache`, `cache-clean`)
- Follow existing patterns for consistency
- Use dashes for multi-word commands (`delete-all` not `deleteall`)

### Commit Message Guidelines

We follow conventional commit format:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test-related changes
- `refactor:` for code refactoring
- `chore:` for maintenance tasks

Examples:
- `feat: add support for environment variable export`
- `fix: resolve memory leak in GUI component`
- `docs: update installation instructions`
- `feat: add --project flag for better CLI consistency`
- `refactor: rename 'sets' to 'projects' throughout codebase`

## Testing

### Running Tests

```bash
# Run all tests (includes unit and integration tests)
cargo test

# Run comprehensive integration tests
cargo test --test integration_tests

# Run specific integration test modules
cargo test --test cli_basic
cargo test --test cli_export_run  
cargo test --test cli_setup_launch

# Run with environment overrides for testing
SAFEHOLD_HOME=/tmp/test cargo test

# Test PATH functionality in dry-run mode
SAFEHOLD_PATH_DRY_RUN=1 cargo test
```

### Test Coverage

SafeHold has comprehensive test coverage across multiple areas:

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test complete CLI workflows with isolated environments
  - `integration_tests.rs`: Comprehensive tests covering:
    - Version information and help system
    - Complete project lifecycle (create, list, delete)
    - Full credential management (add, get, update, delete, list)
    - Global credentials functionality
    - Count and statistics features
    - Export functionality
    - **Maintenance operations**: Cache cleaning and about command
    - **Destructive operations**: Delete-all command with safety testing
    - **Force flag functionality**: Testing bypass mechanisms
    - **Command aliases**: Testing all command aliases and shortcuts
    - Error handling and edge cases
    - Cross-platform compatibility
  - `cli_basic.rs`: Basic CLI operations (create, add, get, list, delete)
  - `cli_export_run.rs`: Export and run functionality
  - `cli_setup_launch.rs`: Setup with `--add-path` and GUI launch testing
- **GUI tests**: Manual testing of GUI features (automated GUI testing is complex)
- **Cross-platform**: All tests designed to work on Windows, macOS, and Linux

### Adding New Tests

When contributing new features:

1. **Add unit tests** for new functions in the same file using `#[cfg(test)]`
2. **Add integration tests** to `tests/integration_tests.rs` for new CLI commands
3. **Update existing tests** if modifying existing functionality
4. **Test cross-platform** compatibility on different operating systems
5. **Test destructive operations safely** using isolated test environments

### Testing Destructive Commands

⚠️ **IMPORTANT**: When testing destructive operations like `delete-all`:

- **ALWAYS use isolated test environments** with `SAFEHOLD_HOME` set to temporary directories
- **NEVER run destructive tests** on your actual SafeHold data
- **Use `--force` flags** in tests to bypass confirmations
- **Verify cleanup** after destructive operations
- **Test both confirmation and bypass mechanisms**

Example of safe destructive testing:
```rust
#[test]
fn test_delete_all_command_safety() -> Result<()> {
    let env = TestEnv::new()?; // Creates isolated environment
    
    // Create test data
    env.run_success(&["create", "test-project"])?;
    env.run_success(&["add", "--project", "test-project", "--key", "TEST", "--value", "data"])?;
    
    // Test destructive operation with force flag (safe in isolated env)
    let output = env.run_success(&["delete-all", "--force"])?;
    assert!(output.contains("permanently deleted"));
    
    Ok(())
}
```

### Test Environment Isolation

Integration tests use `SAFEHOLD_HOME` environment variable to create isolated test environments that don't interfere with your actual SafeHold data.

### Testing Different Build Modes

```bash
# Test CLI-only build
cargo build
cargo test

# Test GUI build
cargo build --features gui
cargo test

# Test GUI launch (requires GUI build)
./target/debug/safehold launch --gui
```

### Adding Tests

- Add unit tests for new functions
- Add integration tests for new CLI commands
- Ensure tests work with both CLI and GUI builds
- Use temporary directories for file operations
- Test error conditions and edge cases

## Code Style

### Rust Style

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html)
- Use `cargo fmt` to format your code
- Use `cargo clippy` for linting
- Follow naming conventions:
  - `snake_case` for variables, functions, and modules
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Project-Specific Guidelines

- **Error handling**: Use `anyhow` for application errors, `thiserror` for library errors
- **Security**: Zeroize sensitive data after use
- **Documentation**: Document public APIs and functions with comprehensive docstrings using `///` comments
  - Include parameter descriptions
  - Document return values and error conditions
  - Provide examples where helpful
- **Feature flags**: Use feature flags appropriately (CLI vs GUI). The GUI must compile and run only when the `gui` feature is enabled; otherwise `safehold launch --gui` should print a helpful reinstall hint.
- **Cross-platform**: Ensure code works on Windows, macOS, and Linux
- **PATH updates**: `safehold setup --add-path` attempts to add Cargo's bin directory to PATH. For tests and local development safety, respect `SAFEHOLD_PATH_DRY_RUN=1` to avoid mutating your environment.
- **Terminology**: Use "projects" instead of "sets" in user-facing text, help messages, and documentation for better clarity
- **CLI flags**: Use `--project` (short: `-p`) for consistency with the GUI
- **Data safety**: Never delete or modify user data during updates. Data is stored separately in `~/.safehold/` and should be preserved across installations.
- **CLI output formatting**: Ensure proper spacing between emojis and text in success/error messages

### Code Organization

```
src/
├── main.rs              # Entry point and application setup with async main
├── core/                # Core functionality modules
│   ├── mod.rs          # Core module declarations
│   ├── config.rs       # Configuration management and file paths
│   │                   # - Handles SAFEHOLD_HOME environment variable
│   │                   # - Manages app configuration and directory structure
│   │                   # - Version compatibility checking and migration
│   ├── crypto.rs       # Encryption/decryption logic with AES-256-GCM
│   └── store.rs        # Low-level storage operations for projects and credentials
│                       # - GUI/CLI launch detection and routing
├── cli/                # CLI interface modules
│   ├── mod.rs          # CLI module declarations
│   ├── cli.rs          # CLI argument parsing and command dispatch
│   │                   # - Contains all command definitions and argument structures
│   │                   # - Includes comprehensive help text and command aliases
│   │                   # - Dispatches to appropriate functions in operations
│   │                   # - Async command dispatch for update checking
│   └── styles.rs       # Output formatting and styling
│                       # - Terminal colors and spinner animations
│                       # - Success, error, and warning message formatting
│                       # - Progress indicators and visual feedback
├── gui/                # GUI interface modules
│   ├── mod.rs          # GUI module declarations
│   └── ui.rs           # GUI implementation (feature-gated with 'gui')
│                       # - Main application window and tabs
│                       # - Global credentials tab
│                       # - Settings display with version/author info
│                       # - Project and credential management dialogs
│                       # - Modal error and warning dialogs
│                       # - Update notification integration
├── operations/         # Business logic operations
│   ├── mod.rs          # Operations module declarations
│   ├── envops.rs       # High-level environment variable operations
│   │                   # - All CLI command implementations (cmd_* functions)
│   │                   # - Project CRUD operations
│   │                   # - Global credential management
│   │                   # - Count and statistics functionality
│   │                   # - Comprehensive docstrings for all functions
│   └── master_lock.rs  # Global Master Lock functionality
│                       # - Unified password protection for ALL projects
│                       # - Master password validation and management
│                       # - Global security state management
└── utils/              # Utility modules
    ├── mod.rs          # Utils module declarations
    ├── app_settings.rs # Application settings management
    │                   # - Persistent GUI and CLI preferences storage
    │                   # - Security settings configuration
    │                   # - Session and interface preferences
    ├── install.rs      # Installation and setup functionality
    └── update_checker.rs # Update checking functionality
                        # - Async update checking from crates.io
                        # - Version comparison and notification
                        # - CLI check-update command implementation
                        # - GUI update notification integration

tests/
├── integration_tests.rs # Comprehensive integration test suite
├── cli_basic.rs         # Basic CLI functionality tests
├── cli_export_run.rs    # Export and run command tests
└── cli_setup_launch.rs  # Setup and launch command tests
```

## Submitting Changes

### Pull Request Process

1. **Ensure your PR**:
   - Has a clear title and description
   - References any related issues
   - Includes tests for new functionality
   - Passes all existing tests
   - Follows code style guidelines

2. **PR Description** should include:
   - What changes were made
   - Why the changes were needed
   - How to test the changes
   - Any breaking changes

3. **Wait for review**:
   - Address any feedback from maintainers
   - Make requested changes
   - Keep your branch updated with main

### Review Process

- Maintainers will review your PR
- Automated checks (tests, linting) must pass
- At least one maintainer approval required
- CI/CD pipeline will run tests on multiple platforms

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- **Clear title** describing the issue
- **Steps to reproduce** the problem
- **Expected behavior** vs actual behavior
- **Environment details**:
  - OS and version
  - Rust version (`rustc --version`)
  - SafeHold version
- **Error messages** or logs
- **Screenshots** for GUI issues

### Feature Requests

For feature requests:

- **Clear description** of the proposed feature
- **Use case** or problem it solves
- **Proposed implementation** if you have ideas
- **Alternatives considered**

### Security Issues

For security-related issues:

- **DO NOT** create public issues
- Email security concerns to:  [contact@muhammadfiaz.com](mailto:contact@muhammadfiaz.com)
- Include detailed information about the vulnerability

### General Contact

For questions, support, or feedback:
- **Email**: contact@muhammadfiaz.com
- **GitHub Issues**: [Create an issue](https://github.com/muhammad-fiaz/safehold/issues)

## Recognition

Contributors will be:
- Listed in CHANGELOG.md for significant contributions
- Acknowledged in release notes
- Added to a future contributors file

Thank you for contributing to SafeHold! 🚀