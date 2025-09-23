//! Environment operations: add/get/list/delete/export/run/show/clean
use crate::cli::{
    CountArgs, ExportArgs, GlobalKeyArgs, GlobalKeyValueArgs, ProjectKeyArgs, ProjectKeyValueArgs,
    ProjectTargetArgs, RunArgs,
};
use crate::config::{self, env_enc_path, lock_path};
use crate::crypto::{self, LockInfo};
use anyhow::{Result, bail};
// use dotenvy; // parsing implemented manually
use crate::styles;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Parse dotenv-style bytes into a sorted map.
fn read_env_map_from_bytes(bytes: &[u8]) -> Result<BTreeMap<String, String>> {
    // Parse as dotenv lines
    let s = String::from_utf8_lossy(bytes);
    let mut map = BTreeMap::new();
    for line in s.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

/// Serialize map to dotenv-style string.
fn write_env_string(map: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(&format!("{}={}\n", k, v));
    }
    out
}

/// Resolve a project directory from id or name; supports "global".
fn resolve_set_dir(id_or_name: &str) -> Result<PathBuf> {
    let cfg = config::load_config()?;
    if id_or_name == "global" {
        return config::global_dir();
    }
    // accept name or id
    if let Some(s) = cfg
        .sets
        .iter()
        .find(|s| s.id == id_or_name || s.name == id_or_name)
    {
        return config::set_dir(&s.id);
    }
    bail!("project not found: {}", id_or_name)
}

/// Load encryption key for dir (password-derived if locked, else app key).
/// Uses SAFEHOLD_PASSWORD if set. Checks Global Master Lock first.
fn load_key_for_dir(dir: &PathBuf) -> Result<[u8; 32]> {
    let base = config::base_dir()?;

    // Check Global Master Lock first
    if crate::master_lock::is_master_lock_enabled() {
        // Global Master Lock is enabled - use master password for ALL projects
        let master_password = match std::env::var("SAFEHOLD_MASTER_PASSWORD") {
            Ok(p) => p,
            Err(_) => {
                crate::styles::info("🔒 Global Master Lock is active");
                rpassword::prompt_password("Master Password: ")?
            }
        };

        // Verify master password
        if !crate::master_lock::verify_master_password(&master_password)? {
            anyhow::bail!("❌ Invalid master password");
        }

        // Use master password to derive key for this project
        // We need a consistent salt for master lock, so use a fixed one
        let master_salt = b"safehold_master_lock_salt_v1";
        let key = crypto::derive_key_from_password_and_salt(&master_password, master_salt)?;
        return Ok(key);
    }

    // Standard individual project locking
    let lock_path = lock_path(dir);
    if lock_path.exists() {
        let lock: LockInfo = serde_json::from_slice(&fs::read(&lock_path)?)?;
        let password = match std::env::var("SAFEHOLD_PASSWORD") {
            Ok(p) => p,
            Err(_) => rpassword::prompt_password("Password: ")?,
        };
        let key = crypto::derive_key_from_password(&password, &lock)?;
        Ok(key)
    } else {
        crypto::load_app_key(&base)
    }
}

/// Decrypt and read env map from a project directory.
fn read_env_map(dir: &PathBuf) -> Result<BTreeMap<String, String>> {
    let key = load_key_for_dir(dir)?;
    let enc = fs::read(env_enc_path(dir)).unwrap_or_default();
    if enc.is_empty() {
        return Ok(BTreeMap::new());
    }
    let pt = crypto::decrypt_with_key(&key, &enc)?;
    read_env_map_from_bytes(&pt)
}

/// Encrypt and write env map to a project directory.
fn write_env_map(dir: &PathBuf, map: &BTreeMap<String, String>) -> Result<()> {
    let key = load_key_for_dir(dir)?;
    let s = write_env_string(map);
    let ct = crypto::encrypt_with_key(&key, s.as_bytes())?;
    fs::write(env_enc_path(dir), ct)?;
    Ok(())
}

/// Add or replace a key/value in a project. Reads value from stdin if not provided.
pub fn cmd_add(args: ProjectKeyValueArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.project)?;
    let mut map = read_env_map(&dir)?;
    let value = match args.value {
        Some(v) => v,
        None => {
            // read from stdin
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim_end().to_string()
        }
    };
    map.insert(args.key, value);
    write_env_map(&dir, &map)?;
    styles::ok("Added");
    Ok(())
}

/// Print a single value for the given key.
pub fn cmd_get(args: ProjectKeyArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.project)?;
    let map = read_env_map(&dir)?;
    if let Some(v) = map.get(&args.key) {
        println!("{}", v);
    } else {
        bail!("key not found");
    }
    Ok(())
}

/// List all key=value pairs in a project.
pub fn cmd_list(args: ProjectTargetArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.project)?;
    let map = read_env_map(&dir)?;
    for (k, v) in map {
        println!("{}={}", k, v);
    }
    Ok(())
}

/// Delete a key in a project (no-op if missing).
pub fn cmd_delete(args: ProjectKeyArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.project)?;
    let mut map = read_env_map(&dir)?;
    if map.remove(&args.key).is_some() {
        write_env_map(&dir, &map)?;
        styles::ok("Deleted");
    } else {
        styles::warn("Key not found");
    }
    Ok(())
}

/// Export a project or global into a .env file; supports temp mode and overwrite.
pub fn cmd_export(args: ExportArgs) -> Result<()> {
    let dir = if args.global {
        config::global_dir()?
    } else {
        let project = args
            .project
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("--project or --global required"))?;
        resolve_set_dir(project)?
    };
    let pb = styles::spinner("Decrypting and writing .env...");
    let map = read_env_map(&dir)?;
    let filename = args.file.unwrap_or_else(|| ".env".into());
    if std::path::Path::new(&filename).exists() && !args.force {
        bail!("{} exists, use --force", filename);
    }
    let content = write_env_string(&map);
    fs::write(&filename, content)?;
    if args.temp {
        // best-effort delete on exit
        let name = filename.clone();
        ctrlc::set_handler(move || {
            let _ = std::fs::remove_file(&name);
            std::process::exit(0);
        })
        .ok();
        styles::info(format!(
            "Temporary .env written: {} (will delete on Ctrl+C or process exit)",
            filename
        ));
    } else {
        styles::ok(format!(".env written: {}", filename));
    }
    styles::finish_spinner(pb, "Done");
    Ok(())
}

/// Run a program with environment variables injected from a project and optionally from global.
pub fn cmd_run(args: RunArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.project)?;
    let mut map = read_env_map(&dir)?;
    if args.with_global {
        let gdir = config::global_dir()?;
        let gmap = read_env_map(&gdir)?;
        map.extend(gmap);
    }
    // Prepare command
    let mut iter = args.command.iter();
    let prog = iter.next().expect("command required");
    let mut cmd = std::process::Command::new(prog);
    cmd.args(iter);
    cmd.envs(map);
    let status = cmd.status()?;
    if !status.success() {
        bail!("process exited with status {:?}", status.code());
    }
    Ok(())
}

/// Show all projects and their keys to stdout.
pub fn cmd_show_all() -> Result<()> {
    let cfg = config::load_config()?;
    styles::info("GLOBAL:");
    let gdir = config::global_dir()?;
    if let Ok(map) = read_env_map(&gdir) {
        for (k, v) in map {
            println!("  {}={}", k, v);
        }
    }
    for s in cfg.sets {
        styles::info(format!("PROJECT {} ({})", s.id, s.name));
        let dir = config::set_dir(&s.id)?;
        if let Ok(map) = read_env_map(&dir) {
            for (k, v) in map {
                println!("  {}={}", k, v);
            }
        }
    }
    Ok(())
}

/// Recursively remove plaintext .env files in current directory tree.
pub fn cmd_clean() -> Result<()> {
    let mut removed = 0usize;
    for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.file_name().to_string_lossy() == ".env" {
            // delete file
            let _ = fs::remove_file(entry.path());
            removed += 1;
        }
    }
    styles::success(format!("🧹 Removed {} .env files", removed));
    Ok(())
}

/// Update/modify a credential value in a project.
///
/// This function allows updating an existing credential's value within a specific project.
/// If the key doesn't exist, it will return an error. The value can be provided via
/// command line argument or will be prompted securely from stdin if omitted.
///
/// # Arguments
/// * `args` - Contains the project identifier, key name, and optional new value
///
/// # Returns
/// * `Result<()>` - Success or error if the key doesn't exist or operation fails
pub fn cmd_update(args: ProjectKeyValueArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.project)?;
    let mut map = read_env_map(&dir)?;

    // Check if key exists
    if !map.contains_key(&args.key) {
        bail!(
            "❌ Key '{}' not found in project '{}'",
            args.key,
            args.project
        );
    }

    let value = match args.value {
        Some(v) => v,
        None => {
            use std::io::{self, Write};
            print!("🔑 Enter new value for '{}': ", args.key);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    map.insert(args.key.clone(), value);
    write_env_map(&dir, &map)?;
    styles::success(format!(
        "Updated credential '{}' in project '{}'",
        args.key, args.project
    ));
    Ok(())
}

/// Count credentials in projects with various display options.
///
/// This function provides comprehensive statistics about credential storage:
/// - Count for a specific project when `project` is provided
/// - Count for all projects when no project is specified
/// - Optional inclusion of global credentials in the total
/// - Detailed breakdown showing per-project counts when `detailed` is true
///
/// # Arguments
/// * `args` - Contains optional project filter, global inclusion flag, and detail level
///
/// # Returns
/// * `Result<()>` - Success or error if projects cannot be accessed
pub fn cmd_count(args: CountArgs) -> Result<()> {
    let cfg = config::load_config()?;

    if let Some(project) = args.project {
        // Count for specific project
        let dir = resolve_set_dir(&project)?;
        let map = read_env_map(&dir)?;
        let count = map.len();
        styles::info(format!(
            "📊 Project '{}' has {} credential(s)",
            project, count
        ));
    } else {
        // Count for all projects
        let mut total = 0;
        let mut global_count = 0;

        // Count global credentials
        if args.include_global {
            let gdir = config::global_dir()?;
            if let Ok(map) = read_env_map(&gdir) {
                global_count = map.len();
                total += global_count;
            }
        }

        // Count project credentials
        let mut project_counts = Vec::new();
        for s in &cfg.sets {
            let dir = config::set_dir(&s.id)?;
            if let Ok(map) = read_env_map(&dir) {
                let count = map.len();
                total += count;
                project_counts.push((s.name.clone(), s.id.clone(), count));
            }
        }

        styles::header("📊 Credential Count Summary");
        if args.include_global {
            styles::info(format!("🌍 Global: {} credential(s)", global_count));
            styles::divider();
        }

        if args.detailed {
            for (name, _id, count) in project_counts {
                styles::bullet(format!("📁 {}: {} credential(s)", name, count));
            }
            styles::divider();
        }

        styles::success(format!(
            "📊 Total: {} credential(s) across {} project(s)",
            total,
            cfg.sets.len()
        ));
    }
    Ok(())
}

/// Add a key/value credential to global storage.
///
/// Global credentials are accessible from any project and provide a way to store
/// commonly used credentials that don't belong to a specific project context.
/// The value can be provided via command line or prompted securely from stdin.
///
/// # Arguments
/// * `args` - Contains the key name and optional value
///
/// # Returns
/// * `Result<()>` - Success or error if global storage cannot be accessed
pub fn cmd_global_add(args: GlobalKeyValueArgs) -> Result<()> {
    let dir = config::global_dir()?;
    let mut map = read_env_map(&dir)?;

    let value = match args.value {
        Some(v) => v,
        None => {
            use std::io::{self, Write};
            print!("🔑 Enter value for '{}': ", args.key);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    map.insert(args.key.clone(), value);
    write_env_map(&dir, &map)?;
    styles::success(format!("Added global credential '{}'", args.key));
    Ok(())
}

/// Get a credential value from global storage.
///
/// Retrieves and displays the value of a specific credential from global storage.
/// This is useful for accessing commonly used credentials that are shared across projects.
///
/// # Arguments
/// * `args` - Contains the key name to retrieve
///
/// # Returns
/// * `Result<()>` - Success with value printed to stdout, or error if key not found
pub fn cmd_global_get(args: GlobalKeyArgs) -> Result<()> {
    let dir = config::global_dir()?;
    let map = read_env_map(&dir)?;

    match map.get(&args.key) {
        Some(value) => {
            println!("{}", value);
            Ok(())
        }
        None => {
            bail!("❌ Key '{}' not found in global storage", args.key);
        }
    }
}

/// List all credentials in global storage.
///
/// Displays all key-value pairs stored in the global credential storage with
/// a formatted header and total count. Global credentials are accessible from
/// any project context and provide shared credential storage.
///
/// # Returns
/// * `Result<()>` - Success or error if global storage cannot be accessed
pub fn cmd_global_list() -> Result<()> {
    let dir = config::global_dir()?;
    let map = read_env_map(&dir)?;

    if map.is_empty() {
        styles::info("🌍 No global credentials found");
        return Ok(());
    }

    styles::header("🌍 Global Credentials");
    let count = map.len();
    for (k, v) in &map {
        println!("{}={}", k, v);
    }
    styles::info(format!("📊 Total: {} global credential(s)", count));
    Ok(())
}

/// Delete a credential from global storage.
///
/// Removes a specific credential key-value pair from global storage.
/// This operation is permanent and cannot be undone. The function will
/// return an error if the specified key doesn't exist.
///
/// # Arguments
/// * `args` - Contains the key name to delete
///
/// # Returns
/// * `Result<()>` - Success or error if key not found or deletion fails
pub fn cmd_global_delete(args: GlobalKeyArgs) -> Result<()> {
    let dir = config::global_dir()?;
    let mut map = read_env_map(&dir)?;

    if map.remove(&args.key).is_none() {
        bail!("❌ Key '{}' not found in global storage", args.key);
    }

    write_env_map(&dir, &map)?;
    styles::success(format!("Deleted global credential '{}'", args.key));
    Ok(())
}

/// Update/modify a credential value in global storage.
///
/// Updates an existing credential's value in global storage. This function
/// requires the key to already exist - use `cmd_global_add` for new credentials.
/// The new value can be provided via command line or prompted securely from stdin.
///
/// # Arguments
/// * `args` - Contains the key name and optional new value
///
/// # Returns
/// * `Result<()>` - Success or error if key doesn't exist or update fails
pub fn cmd_global_update(args: GlobalKeyValueArgs) -> Result<()> {
    let dir = config::global_dir()?;
    let mut map = read_env_map(&dir)?;

    // Check if key exists
    if !map.contains_key(&args.key) {
        bail!("❌ Key '{}' not found in global storage", args.key);
    }

    let value = match args.value {
        Some(v) => v,
        None => {
            use std::io::{self, Write};
            print!("🔑 Enter new value for '{}': ", args.key);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    map.insert(args.key.clone(), value);
    write_env_map(&dir, &map)?;
    styles::success(format!("Updated global credential '{}'", args.key));
    Ok(())
}

/// Helper function to ask for user confirmation for destructive operations
///
/// # Arguments
/// * `message` - The confirmation message to display
/// * `force` - If true, skip confirmation and return true
///
/// # Returns
/// * `bool` - True if confirmed, false if cancelled
fn ask_confirmation(message: &str, force: bool) -> bool {
    if force {
        return true;
    }

    use std::io::{self, Write};

    println!();
    styles::warn(message);
    print!("🤔 Type 'yes' to continue or anything else to cancel: ");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_lowercase() == "yes",
        Err(_) => false,
    }
}

/// Clean cache and temporary files
///
/// This command removes SafeHold cache files and temporary data but preserves
/// all credential projects and data.
///
/// # Arguments
/// * `force` - Skip confirmation prompt if true
///
/// # Returns
/// * `Result<()>` - Success or error if cleanup fails
pub fn cmd_clean_cache(force: bool) -> Result<()> {
    let base_dir = config::base_dir()?;
    let cache_dirs = vec![
        base_dir.join("cache"),
        base_dir.join("tmp"),
        base_dir.join("temp"),
        base_dir.join(".cache"),
    ];

    // Count files to be cleaned
    let mut total_files = 0;
    let mut total_size = 0u64;

    for cache_dir in &cache_dirs {
        if cache_dir.exists() {
            for entry in WalkDir::new(cache_dir).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    total_files += 1;
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        }
    }

    if total_files == 0 {
        styles::info("✨ No cache files found to clean");
        return Ok(());
    }

    let size_mb = total_size as f64 / 1024.0 / 1024.0;
    let message = format!(
        "🗑️ This will delete {} cache files ({:.2} MB)\n⚠️ This action cannot be undone, but your credentials will NOT be affected.",
        total_files, size_mb
    );

    if !ask_confirmation(&message, force) {
        styles::info("❌ Cache cleaning cancelled");
        return Ok(());
    }

    // Perform cleanup
    let mut cleaned_files = 0;
    let mut cleaned_size = 0u64;

    for cache_dir in &cache_dirs {
        if cache_dir.exists() {
            for entry in WalkDir::new(cache_dir).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        cleaned_size += metadata.len();
                    }
                    if fs::remove_file(entry.path()).is_ok() {
                        cleaned_files += 1;
                    }
                }
            }
            // Try to remove empty directories
            let _ = fs::remove_dir_all(cache_dir);
        }
    }

    let cleaned_mb = cleaned_size as f64 / 1024.0 / 1024.0;
    styles::success(format!(
        "✅ Cleaned {} files ({:.2} MB) from cache",
        cleaned_files, cleaned_mb
    ));
    Ok(())
}

/// Delete ALL projects and data (VERY DESTRUCTIVE!)
///
/// This command completely removes all SafeHold data including:
/// - All credential projects
/// - Global credentials  
/// - Configuration files
/// - Cache and temporary files
///
/// # Arguments
/// * `force` - Skip confirmation prompt if true (DANGEROUS!)
///
/// # Returns
/// * `Result<()>` - Success or error if deletion fails
pub fn cmd_delete_all(force: bool) -> Result<()> {
    let base_dir = config::base_dir()?;

    if !base_dir.exists() {
        styles::info("✨ No SafeHold data found to delete");
        return Ok(());
    }

    // Count all data
    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut total_size = 0u64;

    for entry in WalkDir::new(&base_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            total_files += 1;
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        } else if entry.file_type().is_dir() && entry.path() != base_dir {
            total_dirs += 1;
        }
    }

    let size_mb = total_size as f64 / 1024.0 / 1024.0;

    println!();
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│                        🚨 DANGER ZONE 🚨                        │");
    println!("│                                                                 │");
    println!("│  This will PERMANENTLY DELETE ALL SafeHold data:               │");
    println!("│  • All credential projects and their encrypted data            │");
    println!("│  • Global credentials                                          │");
    println!("│  • Configuration files                                         │");
    println!("│  • Cache and temporary files                                   │");
    println!("│                                                                 │");
    println!(
        "│  📊 Data to be deleted: {} files, {} directories ({:.2} MB)     │",
        total_files, total_dirs, size_mb
    );
    println!("│                                                                 │");
    println!("│  ⚠️ THIS ACTION CANNOT BE UNDONE!                              │");
    println!("│  ⚠️ ALL YOUR CREDENTIALS WILL BE LOST!                         │");
    println!("│                                                                 │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    if !force {
        use std::io::{self, Write};

        println!();
        styles::error("🚨 FINAL WARNING: This will delete ALL your SafeHold data!");
        print!("💀 Type 'DELETE ALL MY DATA' to continue: ");
        io::stdout().flush().unwrap_or(());

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() != "DELETE ALL MY DATA" {
                    styles::info("❌ Deletion cancelled - data preserved");
                    return Ok(());
                }
            }
            Err(_) => {
                styles::info("❌ Deletion cancelled - data preserved");
                return Ok(());
            }
        }
    }

    // Perform complete deletion
    match fs::remove_dir_all(&base_dir) {
        Ok(()) => {
            styles::success("💥 All SafeHold data has been permanently deleted");
            styles::info("ℹ️ Run 'safehold setup' to reinitialize SafeHold");
        }
        Err(e) => {
            bail!("❌ Failed to delete data: {}", e);
        }
    }

    Ok(())
}

/// Show application information and details
///
/// Displays comprehensive information about SafeHold including:
/// - Version and build information
/// - Developer and repository details
/// - Features and capabilities
/// - System information
///
/// # Returns
/// * `Result<()>` - Success or error if information cannot be retrieved
pub fn cmd_about() -> Result<()> {
    let base_dir = config::base_dir().unwrap_or_else(|_| PathBuf::from("Not configured"));
    let config_exists = config::base_dir().map(|p| p.exists()).unwrap_or(false);

    // Count projects and credentials
    let (total_projects, total_credentials) = if config_exists {
        match config::load_config() {
            Ok(cfg) => {
                let projects = cfg.sets.len();
                let mut credentials = 0;

                // Count global credentials
                if let Ok(global_dir) = config::global_dir() {
                    if let Ok(map) = read_env_map(&global_dir) {
                        credentials += map.len();
                    }
                }

                // Count project credentials
                for project in &cfg.sets {
                    if let Ok(project_dir) = config::set_dir(&project.id) {
                        if let Ok(map) = read_env_map(&project_dir) {
                            credentials += map.len();
                        }
                    }
                }

                (projects, credentials)
            }
            Err(_) => (0, 0),
        }
    } else {
        (0, 0)
    };

    println!();
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  ███████╗ █████╗ ███████╗███████╗██╗  ██╗ ██████╗ ██╗     ██████╗ │");
    println!("│  ██╔════╝██╔══██╗██╔════╝██╔════╝██║  ██║██╔═══██╗██║     ██╔══██╗│");
    println!("│  ███████╗███████║█████╗  █████╗  ███████║██║   ██║██║     ██║  ██║│");
    println!("│  ╚════██║██╔══██║██╔══╝  ██╔══╝  ██╔══██║██║   ██║██║     ██║  ██║│");
    println!("│  ███████║██║  ██║██║     ███████╗██║  ██║╚██████╔╝███████╗██████╔╝│");
    println!("│  ╚══════╝╚═╝  ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═════╝ │");
    println!("│                                                                     │");
    println!("│                🔐 Professional Credential Manager 🔐               │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    println!("📋 Application Information:");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("   Name: {}", env!("CARGO_PKG_NAME"));
    println!("   Description: {}", env!("CARGO_PKG_DESCRIPTION"));
    println!();

    println!("👨‍💻 Developer Information:");
    println!("   Author: {}", env!("CARGO_PKG_AUTHORS"));
    println!("   Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    println!("   Homepage: {}", env!("CARGO_PKG_HOMEPAGE"));
    println!("   License: {}", env!("CARGO_PKG_LICENSE"));
    println!();

    println!("🔧 Build Information:");
    println!("   Rust Version: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!("   Target: {}", std::env::consts::ARCH);
    println!("   OS: {}", std::env::consts::OS);
    #[cfg(feature = "gui")]
    println!("   GUI Support: ✅ Enabled");
    #[cfg(not(feature = "gui"))]
    println!("   GUI Support: ❌ Disabled");
    println!();

    println!("📊 Current Status:");
    println!(
        "   Configuration: {}",
        if config_exists {
            "✅ Initialized"
        } else {
            "❌ Not configured"
        }
    );
    println!("   Base Directory: {}", base_dir.display());
    println!("   Total Projects: {}", total_projects);
    println!("   Total Credentials: {}", total_credentials);
    println!();

    println!("🔐 Security Features:");
    println!("   • AES-256-GCM encryption for data at rest");
    println!("   • Argon2id password hashing with salt");
    println!("   • Memory-safe Rust implementation");
    println!("   • Cross-platform secure key derivation");
    println!("   • No plaintext storage of sensitive data");
    println!();

    println!("🚀 Available Features:");
    println!("   • Multi-project credential management");
    println!("   • Global credential storage");
    println!("   • CLI and GUI interfaces");
    println!("   • Environment variable export");
    println!("   • Command execution with injected credentials");
    println!("   • Cross-platform compatibility");
    println!("   • Comprehensive backup and restore");
    println!();

    println!("📚 Quick Commands:");
    println!("   safehold create <project>     - Create new project");
    println!("   safehold add <project> <key>  - Add credential");
    println!("   safehold list <project>       - List credentials");
    println!("   safehold global-add <key>     - Add global credential");
    println!("   safehold export <project>     - Export to .env");
    println!("   safehold --help               - Show all commands");
    println!();

    if !config_exists {
        styles::warn("⚠️ SafeHold is not yet configured. Run 'safehold setup' to get started!");
    }

    Ok(())
}
