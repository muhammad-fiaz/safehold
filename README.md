<div align="center">

# SafeHold - Environment Variable Manager

</div>

<div align="center">

[![GitHub](https://img.shields.io/badge/GitHub-muhammad--fiaz/safehold-blue)](https://github.com/muhammad-fiaz/safehold)
[![Crates.io](https://img.shields.io/crates/v/safehold)](https://crates.io/crates/safehold)
[![GitHub release](https://img.shields.io/github/v/release/muhammad-fiaz/safehold)](https://github.com/muhammad-fiaz/safehold/releases)
[![Docs.rs](https://docs.rs/safehold/badge.svg)](https://docs.rs/safehold)
[![License](https://img.shields.io/github/license/muhammad-fiaz/safehold)](https://github.com/muhammad-fiaz/safehold/blob/main/LICENSE)
[![GitHub last commit](https://img.shields.io/github/last-commit/muhammad-fiaz/safehold)](https://github.com/muhammad-fiaz/safehold)
[![GitHub issues](https://img.shields.io/github/issues/muhammad-fiaz/safehold)](https://github.com/muhammad-fiaz/safehold/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/muhammad-fiaz/safehold)](https://github.com/muhammad-fiaz/safehold/pulls)
[![Crates.io recent downloads](https://img.shields.io/crates/d/safehold)](https://crates.io/crates/safehold)
[![CodeQL](https://github.com/muhammad-fiaz/safehold/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/muhammad-fiaz/safehold/actions/workflows/github-code-scanning/codeql)
[![Dependabot Updates](https://github.com/muhammad-fiaz/safehold/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/muhammad-fiaz/safehold/actions/workflows/dependabot/dependabot-updates)
[![Donate](https://img.shields.io/badge/Donate-❤️-green.svg)](https://pay.muhammadfiaz.com)

</div>

SafeHold is a secure, cross-platform environment variable manager with both CLI and GUI interfaces. It stores environment variables and secrets encrypted at rest, supporting unlocked projects (app-managed key) and locked projects (password-protected). Perfect for managing project-specific and global environment variables without exposing sensitive data.

## Features

- **Secure Storage**: All environment variables encrypted with AES-256-GCM or XSalsa20-Poly1305.
- **Flexible Projects**: Create unlocked or password-locked environment variable sets per project.
- **Global Environment Variables**: Manage global credentials outside of projects for shared access.
- **🔐 Global Master Lock**: Unified password protection for ALL projects - when enabled, all projects require the same master password for ultimate security consistency.
- **⚙️ Application Settings**: Persistent GUI and CLI preferences stored separately from project data, including security settings and interface preferences.
- **Full CRUD Operations**: Create, Read, Update, Delete operations for both projects and credentials.
- **Count & Statistics**: Count credentials per project and get detailed statistics.
- **Maintenance Operations**: Clean cache files and perform administrative cleanup with safety confirmations.
- **Developer Information**: Comprehensive about command showing application, security, and developer details.
- **CLI Interface**: Comprehensive command-line operations for all features with intuitive aliases.
- **GUI Interface**: Cross-platform graphical interface with Global tab, master lock controls, and persistent settings.
- **Export & Run**: Decrypt to `.env` files or inject into processes.
- **Clean Up**: Find and remove stray plaintext `.env` files.
- **Cross-Platform**: Works on Windows, macOS, and Linux with comprehensive testing.

## Installation

### Prerequisites
- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/muhammad-fiaz/safehold.git
   cd safehold
   ```

2. Build the CLI version:
   ```bash
   cargo build --release
   ```

3. (Optional) Build with GUI support:
   ```bash
   cargo build --release --features gui
   ```

4. Install locally (CLI only):
   ```bash
   cargo install --path .
   ```

5. Install locally with GUI:
   ```bash
   cargo install --path . --features gui
   ```

6. Add to PATH:
   - **Windows**: `setx PATH "%USERPROFILE%\.cargo\bin;%PATH%"`
   - **Linux/macOS**: Add `export PATH="$HOME/.cargo/bin:$PATH"` to your shell profile (e.g., `~/.bashrc`).

7. Verify installation:
   ```bash
   safehold --help
   ```

## Installation modes: CLI vs GUI

SafeHold can be installed in two modes:

- CLI only (default):
   - `cargo install safehold`
   - Installs the command-line interface only.
- CLI + GUI:
   - `cargo install safehold --features gui`
   - Installs both CLI and the graphical UI.

If you try to launch the GUI without installing it, the CLI will inform you how to reinstall with the GUI feature.

## Updating SafeHold

SafeHold stores all your projects and credentials in a separate data directory (`~/.safehold/` on Linux/macOS, or equivalent on Windows). Updating SafeHold will **never delete or remove your existing data**.

To update to the latest version:

1. Update from crates.io:
   ```bash
   cargo install safehold --features gui  # or without --features for CLI-only
   ```

2. Or update from source:
   ```bash
   git pull
   cargo build --release --features gui
   cargo install --path . --features gui
   ```

Your projects, credentials, and settings will remain intact. The update only replaces the SafeHold executable with the latest version.

**Important**: While data loss is unlikely, it's always good practice to back up your `~/.safehold/` directory before major updates.

## CLI styling options

SafeHold’s CLI supports global switches to control colors and animations:

- `--color <auto|always|never>`: force color usage. Default is `auto` (uses color only when attached to a TTY).
- `--style <fancy|plain>`: choose `fancy` to enable spinner animations and styled prefixes, or `plain` for simple text. Default is `fancy`.
- `--quiet`: suppress non-essential output.

Examples:

- Disable colors and animations: `safehold --color never --style plain list-projects`
- Keep colors but disable spinners: `safehold --style plain create projectA --password secret`
- Quiet mode for scripting: `safehold --quiet export --project projectA --file .env`

## Usage

SafeHold stores data in `~/.safehold/` (or equivalent on Windows/macOS). Run `safehold setup` for initial setup and PATH guidance.

### CLI Commands

#### Project Management
- Create unlocked project: `safehold create <name>` (aliases: `new`)
- Create locked project: `safehold create <name> --lock` (prompts for password) or `safehold create <name> --password <pwd>`
- List projects: `safehold list-projects` (aliases: `ls`, `projects`)
- Delete project: `safehold delete-project <id|name>` (aliases: `rm`, `remove`)

#### Credential Management
- Add key: `safehold add --project <id|name> --key <key> --value <value>` (aliases: `set`)
- Get value: `safehold get --project <id|name> --key <key>` (aliases: `show`)
- Update key: `safehold update --project <id|name> --key <key> --value <value>` (aliases: `modify`, `change`, `edit`)
- List keys: `safehold list --project <id|name>` (aliases: `keys`)
- Delete key: `safehold delete --project <id|name> --key <key>` (aliases: `del`, `rm-key`)
- Count credentials: `safehold count [--project <id|name>] [--detailed] [--include-global]` (aliases: `total`)

#### Global Credentials
- Add global credential: `safehold global-add --key <key> --value <value>` (aliases: `gadd`, `global-set`)
- Get global credential: `safehold global-get --key <key>` (aliases: `gget`, `global-show`)
- Update global credential: `safehold global-update --key <key> --value <value>` (aliases: `gupdate`, `global-modify`)
- List global credentials: `safehold global-list` (aliases: `glist`, `global-keys`)
- Delete global credential: `safehold global-delete --key <key>` (aliases: `gdel`, `global-rm`)

#### Export & Run
- Export to `.env`: `safehold export --project <id|name> [--file <name>] [--force] [--temp]`
- Run with env vars: `safehold run --project <id|name> [--with-global] -- <command>`

#### Utilities
- Show all: `safehold show-all` (prompts for locked sets)
- Clean stray `.env`: `safehold clean`
- Clean cache: `safehold clean-cache [--force]` (aliases: `clear-cache`, `cache-clean`)
- Application info: `safehold about` (aliases: `info`)
- Setup: `safehold setup` prints PATH guidance; `safehold setup --add-path` attempts to add Cargo's bin folder to PATH automatically
- Launch GUI: `safehold launch --gui` launches the GUI when installed with the `gui` feature; otherwise shows a hint to reinstall with GUI

#### 🔐 Security Features
- **Global Master Lock**: `safehold master-lock [--enable|--disable]` (aliases: `mlock`, `global-master`)
  - 🔒 **Enable**: ALL projects require the SAME master password (unified security)
  - 🔓 **Disable**: Projects use individual lock settings (standard security)
  - **Status**: Run without flags to see current master lock status
  - **Environment Variable**: Set `SAFEHOLD_MASTER_PASSWORD` to bypass prompts when master lock is enabled

#### ⚠️ Destructive Operations
- **DELETE ALL DATA**: `safehold delete-all [--force]` (aliases: `clear-all`, `nuke`)
  - ⚠️ **WARNING**: This permanently deletes ALL projects, credentials, and configuration files
  - ⚠️ **CANNOT BE UNDONE**: All data will be lost forever
  - Use `--force` to skip confirmation (DANGEROUS!)
  - Without `--force`, requires typing "DELETE ALL MY DATA" to confirm

### GUI Usage

If built with `--features gui`, launch the graphical interface:

```bash
safehold launch --gui
```

- **Sidebar**: Lists Global and all projects.
- **Main Panel**: Displays decrypted credentials for selected project (prompts for password if locked).
- **Global Tab**: Manage global credentials independent of projects.
- **Settings**: Display version and author information.
- **Actions**: Create projects, add/edit/delete keys, export `.env`, update/modify credentials.

## Examples

1. **Create and populate a project**:
   ```bash
   safehold create project1
   safehold add --project project1 --key GITHUB_TOKEN --value ghp_1234567890abcdef
   safehold add --project project1 --key DB_PASS --value mysecretpassword
   safehold list --project project1
   ```

2. **Update and manage credentials**:
   ```bash
   safehold update --project project1 --key GITHUB_TOKEN --value new_token_value
   safehold count --project project1
   safehold count --detailed --include-global
   ```

3. **Global credentials management**:
   ```bash
   safehold global-add --key COMMON_API_KEY --value shared_secret
   safehold global-list
   safehold global-update --key COMMON_API_KEY --value updated_secret
   ```

4. **Create a locked project**:
   ```bash
   safehold create secureproj --lock
   # Enter password when prompted
   safehold add --project secureproj --key API_KEY --value supersecret
   ```

5. **Export and run**:
   ```bash
   safehold export --project project1 --temp
   safehold run --project project1 --with-global -- cargo run
   ```

6. **Maintenance and information**:
   ```bash
   safehold about  # Show comprehensive application information
   safehold clean-cache  # Clean temporary files (will prompt for confirmation)
   safehold clean-cache --force  # Clean without confirmation
   ```

7. **Global Master Lock Security** 🔐:
   ```bash
   # Check current master lock status
   safehold master-lock
   
   # Enable Global Master Lock (ALL projects will require the same password)
   safehold master-lock --enable
   # Will prompt for master password creation
   
   # Now ALL projects use the master password
   safehold get --project myproject --key API_KEY  # Uses master password
   safehold global-get --key SHARED_KEY           # Uses master password
   
   # Disable Global Master Lock (return to individual project passwords)
   safehold master-lock --disable
   # Will prompt for master password verification
   ```

8. **Emergency cleanup** ⚠️:
   ```bash
   # WARNING: These operations are DESTRUCTIVE and PERMANENT!
   safehold delete-all  # Will prompt for "DELETE ALL MY DATA" confirmation
   safehold delete-all --force  # Bypasses confirmation - USE WITH EXTREME CAUTION!
   ```

4. **Global credentials**:
   ```bash
   safehold create global
   safehold add --project global --key ORG_TOKEN --value sharedtoken
   safehold export --global --file .env --force
   ```

5. **Clean up**:
   ```bash
   safehold clean  # Removes plaintext .env files in current directory tree
   ```

## Application Settings

SafeHold stores persistent application preferences separately from your project data. Settings are automatically saved and restored across sessions.

### GUI Settings
- **Password Visibility**: Control whether passwords are shown by default in the GUI
- **Window Preferences**: Remember window size and position
- **Auto-save**: Configure automatic saving intervals
- **Default Tab**: Set which tab opens by default (Projects/Global)

### CLI Settings  
- **Color Output**: Control colored terminal output (`auto`, `always`, `never`)
- **Output Style**: Choose between `fancy` (with spinners) or `plain` text output
- **Verbose Help**: Show detailed help information by default
- **Destructive Confirmations**: Control confirmation prompts for dangerous operations

### Security Settings
- **Session Timeout**: Automatically lock after inactivity (configurable minutes)
- **Clipboard Security**: Automatically clear copied credentials from clipboard
- **Confirmation Requirements**: Require confirmation for all destructive operations

Settings are stored in `app_settings.json` in your SafeHold data directory and are preserved across updates.

## Security

- **Encryption**: AES-256-GCM for unlocked projects (app-managed key); Argon2id KDF for locked projects.
- **At Rest**: All data encrypted as `.env.enc`; passwords never stored.
- **In Memory**: Sensitive data zeroized after use.
- **Best Practices**: Use locked sets for sensitive data; avoid `--password` in shared shells.

## 🆕 What's New in v0.0.1

SafeHold v0.0.1 brings major enhancements and comprehensive functionality:

### ✨ New Features
- **Full CRUD Operations**: Complete Create, Read, Update, Delete functionality for both projects and credentials
- **Global Credentials**: Manage credentials outside of projects with dedicated global commands
- **Count & Statistics**: New `count` command with detailed project statistics
- **Maintenance Operations**: Cache cleanup and administrative operations with safety confirmations
- **Developer Information**: Comprehensive application details with `about` command
- **Enhanced CLI**: Comprehensive command aliases and improved user experience
- **GUI Enhancements**: Global tab, version display, about dialog, and maintenance operations

### 🔧 New Commands
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

### 🔒 Safety Features
- **Confirmation Prompts**: All destructive operations require confirmation
- **Force Flags**: Bypass confirmations for automation (use with caution)
- **Comprehensive Warnings**: Clear indicators for dangerous operations
- **Data Preservation**: Cache cleaning preserves all credential data

### 🧪 Testing & Quality
- Comprehensive integration test suite with 18+ test scenarios including new features
- Cross-platform compatibility testing (Windows, macOS, Linux)
- Enhanced documentation with detailed docstrings
- Improved error handling and user feedback

### 🎨 User Experience
- Beautiful ASCII art banners for setup and GUI launch
- Fixed emoji spacing in CLI output
- Consistent command structure with intuitive aliases
- Enhanced help system and documentation
- Dynamic version display throughout the application

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for detailed information on how to get started, development setup, and contribution guidelines.

## Contact

For questions, support, or feedback:
- **Email**: contact@muhammadfiaz.com
- **GitHub Issues**: [Create an issue](https://github.com/muhammad-fiaz/safehold/issues)

## License

MIT License. See [LICENSE](LICENSE) for details.
