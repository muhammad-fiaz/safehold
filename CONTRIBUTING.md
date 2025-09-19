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
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture
```

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
```

## How to Contribute

### Types of Contributions

- **Bug fixes**: Fix existing issues
- **Features**: Add new functionality
- **Documentation**: Improve docs, README, examples
- **Tests**: Add or improve test coverage
- **Code quality**: Refactoring, performance improvements

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
# Run all tests
cargo test

# Run integration tests only
cargo test --test cli_basic
cargo test --test cli_export_run
cargo test --test cli_setup_launch

# Run with environment overrides for testing
SAFEHOLD_HOME=/tmp/test cargo test

# Test PATH functionality in dry-run mode
SAFEHOLD_PATH_DRY_RUN=1 cargo test
```

### Test Coverage

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test CLI commands and workflows
  - Basic CLI operations (create, add, get, list, delete)
  - Export and run functionality
  - Setup with `--add-path` (use `SAFEHOLD_PATH_DRY_RUN=1` for safe testing)
  - Launch `--gui` without GUI feature (should show reinstall hint)
- **GUI tests**: Manual testing of GUI features (automated GUI testing is complex)
- **Cross-platform**: Test on Windows, macOS, and Linux

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
- **Documentation**: Document public APIs with `///` comments
- **Feature flags**: Use feature flags appropriately (CLI vs GUI). The GUI must compile and run only when the `gui` feature is enabled; otherwise `safehold launch --gui` should print a helpful reinstall hint.
- **Cross-platform**: Ensure code works on Windows, macOS, and Linux
- **PATH updates**: `safehold setup --add-path` attempts to add Cargo's bin directory to PATH. For tests and local development safety, respect `SAFEHOLD_PATH_DRY_RUN=1` to avoid mutating your environment.
- **Terminology**: Use "projects" instead of "sets" in user-facing text, help messages, and documentation for better clarity
- **CLI flags**: Use `--project` (short: `-p`) for consistency with the GUI
- **Data safety**: Never delete or modify user data during updates. Data is stored separately in `~/.safehold/` and should be preserved across installations.

### Code Organization

```
src/
├── main.rs          # Entry point
├── cli.rs           # CLI argument parsing and dispatch (uses --project flags)
├── config.rs        # Configuration and file paths
├── crypto.rs        # Encryption/decryption logic
├── store.rs         # Project management operations
├── envops.rs        # Environment variable operations (handles project CRUD)
├── ui.rs            # GUI implementation (optional, uses "Projects" terminology)
└── lib.rs           # Library interface (if needed)
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