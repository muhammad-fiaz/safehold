# Changelog

All notable changes to SafeHold will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.2] - 2025-09-23

### Added
- **Modular Architecture**: Restructured project into logical modules (core, cli, gui, operations, utils) for better maintainability
- **Update Checking System**: Automatic update checking from crates.io with internet connectivity detection
- **CLI Update Command**: `check-update` command for manual update checking with semantic version comparison
- **GUI Update Notifications**: Background update checking with modal notification dialogs in GUI
- **Cross-Platform Build System**: Automated build scripts for Windows, macOS, and Linux (Intel/ARM)
- **GitHub Actions CI/CD**: Automated release builds for all supported platforms
- **Release Preparation Tools**: Scripts for automated release preparation and publishing
- **Modal Error/Warning Dialogs**: GUI now displays errors and warnings as modal dialogs instead of text notifications, requiring user acknowledgment
- **Comprehensive Error Handling**: Replaced all `unwrap()` calls with proper error propagation and handling throughout the application
- **Enhanced Delete Operations**: Added confirmation prompts for all delete operations in both CLI and GUI with `--force` flags for automation
- **Improved Error Messages**: Better error handling for corrupted data, missing files, and permission issues
- **Config Migration**: Automatic backward-compatible config migration when updating versions
- **Version Compatibility Check**: Application checks version compatibility and handles upgrades gracefully

### Fixed
- **GUI Project Deletion Bug**: Fixed "file not found" errors when deleting projects that don't have corresponding directories
- **Partial Deletion Handling**: Improved handling of partial deletions and corrupted data states
- **Error Propagation**: All operations now properly handle and report errors instead of panicking
- **Build Warnings**: Eliminated all compiler warnings for cleaner builds

### Changed
- **Project Structure**: Complete reorganization into modular architecture with clear separation of concerns
- **Notification System**: GUI notifications now use modal dialogs for errors/warnings instead of top-panel text
- **Delete Confirmations**: All destructive operations now require explicit confirmation by default
- **Error Display**: Errors are now shown in centered modal dialogs that must be acknowledged
- **Documentation**: Updated README.md and CONTRIBUTING.md to reflect new architecture and features

### Dependencies
- **Added**: `reqwest` v0.12 for HTTP requests to crates.io API
- **Added**: `tokio` v1 with async runtime for background operations
- **Added**: `open` v5 for launching URLs from GUI notifications

### Security
- **Data Safety**: Enhanced validation and error handling prevents data corruption during operations
- **Graceful Degradation**: Application handles corrupted or missing data gracefully without data loss
- **Background Operations**: Update checking runs asynchronously without blocking main operations

## [0.0.1] - 2025-09-23

### Added
- **Full CRUD Operations**: Complete Create, Read, Update, Delete functionality for both projects and credentials
- **Global Credentials**: Manage credentials outside of projects with dedicated global commands
- **Count & Statistics**: New `count` command with detailed project statistics
- **Maintenance Operations**: Cache cleanup and administrative operations with safety confirmations
- **Developer Information**: Comprehensive application details with `about` command
- **Enhanced CLI**: Comprehensive command aliases and improved user experience
- **GUI Enhancements**: Global tab, version display, about dialog, and maintenance operations
- **New Commands**:
  - `update` / `modify` / `change` / `edit` - Update credential values
  - `count` / `total` - Count credentials with various filtering options
  - `global-add` / `gadd` - Add global credentials
  - `global-get` / `gget` - Retrieve global credentials
  - `global-list` / `glist` - List all global credentials
  - `global-update` / `gupdate` - Update global credentials
  - `global-delete` / `gdel` - Delete global credentials
  - `clean-cache` / `clear-cache` / `cache-clean` - Clean temporary files and cache
  - `about` / `info` - Show comprehensive application information
  - `delete-all` / `clear-all` / `nuke` - ⚠️ DESTRUCTIVE: Delete all data permanently

### Security
- **Confirmation Prompts**: All destructive operations require confirmation
- **Force Flags**: Bypass confirmations for automation (use with caution)
- **Comprehensive Warnings**: Clear indicators for dangerous operations
- **Data Preservation**: Cache cleaning preserves all credential data

### Testing
- **Comprehensive Test Suite**: 18+ integration test scenarios including new features
- **Cross-platform Testing**: Compatibility testing for Windows, macOS, and Linux
- **Enhanced Documentation**: Detailed docstrings and improved error handling

### User Experience
- **ASCII Art Banners**: Beautiful banners for setup and GUI launch
- **Fixed Emoji Spacing**: Consistent emoji display in CLI output
- **Command Structure**: Intuitive aliases and consistent command structure
- **Enhanced Help System**: Improved documentation and help information
- **Dynamic Version Display**: Version information throughout the application

### Initial Release
- **Secure Storage**: AES-256-GCM encryption for environment variables
- **Project Management**: Create unlocked or password-locked environment variable sets
- **CLI Interface**: Command-line operations for all features
- **GUI Interface**: Cross-platform graphical interface
- **Export & Run**: Decrypt to `.env` files or inject into processes
- **Cross-Platform**: Works on Windows, macOS, and Linux

## [0.0.0] - 2025-09-19

### Initial Development
- **Project Foundation**: Basic project structure and dependencies
- **Core Architecture**: Initial design and implementation
- **Development Setup**: Build system and testing framework</content>
<parameter name="filePath">e:\Projects\safehold\CHANGELOG.md