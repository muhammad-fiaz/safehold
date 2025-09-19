# SafeHold - Environment Variable Manager

[![GitHub](https://img.shields.io/badge/GitHub-muhammad--fiaz/safehold-blue)](https://github.com/muhammad-fiaz/safehold)

SafeHold is a secure, cross-platform environment variable manager with both CLI and GUI interfaces. It stores environment variables and secrets encrypted at rest, supporting unlocked sets (app-managed key) and locked sets (password-protected). Perfect for managing project-specific and global environment variables without exposing sensitive data.

## Features

- **Secure Storage**: All environment variables encrypted with AES-256-GCM or XSalsa20-Poly1305.
- **Flexible Sets**: Create unlocked or password-locked environment variable sets per project.
- **Global Environment Variables**: Optional global set for shared secrets and config.
- **CLI Interface**: Command-line operations for all features.
- **GUI Interface**: Cross-platform graphical interface (optional build).
- **Export & Run**: Decrypt to `.env` files or inject into processes.
- **Clean Up**: Find and remove stray plaintext `.env` files.
- **Cross-Platform**: Works on Windows, macOS, and Linux.

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

## Usage

SafeHold stores data in `~/.credman/` (or equivalent on Windows/macOS). Run `safehold setup` for initial setup and PATH guidance.

### CLI Commands

#### Set Management
- Create unlocked set: `safehold create <name>`
- Create locked set: `safehold create <name> --lock` (prompts for password) or `safehold create <name> --password <pwd>`
- List sets: `safehold list-sets`
- Delete set: `safehold delete-set <id|name>`

#### Global Credentials
- Create global set: `safehold create global [--lock|--password <pwd>]`
- Export global: `safehold export --global [--file <name>] [--force] [--temp]`

#### Inside a Set
- Add key: `safehold add --set <id|name> --key <key> --value <value>` (or read from stdin if no `--value`)
- Get value: `safehold get --set <id|name> --key <key>`
- List keys: `safehold list --set <id|name>`
- Delete key: `safehold delete --set <id|name> --key <key>`

#### Export & Run
- Export to `.env`: `safehold export --set <id|name> [--file <name>] [--force] [--temp]`
- Run with env vars: `safehold run --set <id|name> [--with-global] -- <command>`

#### Utilities
- Show all: `safehold show-all` (prompts for locked sets)
- Clean stray `.env`: `safehold clean`
- Setup: `safehold setup`
- Launch GUI: `safehold launch --gui` (requires GUI build)

### GUI Usage

If built with `--features gui`, launch the graphical interface:

```bash
safehold launch --gui
```

- **Sidebar**: Lists Global and all sets.
- **Main Panel**: Displays decrypted credentials for selected set (prompts for password if locked).
- **Actions**: Create sets, add/edit/delete keys, export `.env`.

## Examples

1. **Create and populate a project set**:
   ```bash
   safehold create project1
   safehold add --set project1 --key GITHUB_TOKEN --value ghp_1234567890abcdef
   safehold add --set project1 --key DB_PASS --value mysecretpassword
   safehold list --set project1
   ```

2. **Create a locked set**:
   ```bash
   safehold create secureproj --lock
   # Enter password when prompted
   safehold add --set secureproj --key API_KEY --value supersecret
   ```

3. **Export and run**:
   ```bash
   safehold export --set project1 --temp
   safehold run --set project1 --with-global -- cargo run
   ```

4. **Global credentials**:
   ```bash
   safehold create global
   safehold add --set global --key ORG_TOKEN --value sharedtoken
   safehold export --global --file .env --force
   ```

5. **Clean up**:
   ```bash
   safehold clean  # Removes plaintext .env files in current directory tree
   ```

## Security

- **Encryption**: AES-256-GCM for unlocked sets (app-managed key); Argon2id KDF for locked sets.
- **At Rest**: All data encrypted as `.env.enc`; passwords never stored.
- **In Memory**: Sensitive data zeroized after use.
- **Best Practices**: Use locked sets for sensitive data; avoid `--password` in shared shells.

## Contributing

Contributions welcome! Please:
1. Fork the repo.
2. Create a feature branch.
3. Submit a PR with tests.

## License

MIT License. See [LICENSE](LICENSE) for details.

---

For issues or questions, visit [GitHub Issues](https://github.com/muhammad-fiaz/safehold/issues).</content>
<parameter name="filePath">e:\Projects\safehold\README.md