//! Environment operations: add/get/list/delete/export/run/show/clean
use crate::cli::{SetKeyValueArgs, SetKeyArgs, SetTargetArgs, ExportArgs, RunArgs};
use crate::config::{self, env_enc_path, lock_path};
use crate::crypto::{self, LockInfo};
use anyhow::{bail, Result};
// use dotenvy; // parsing implemented manually
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use walkdir::WalkDir;
use crate::styles;

/// Parse dotenv-style bytes into a sorted map.
fn read_env_map_from_bytes(bytes: &[u8]) -> Result<BTreeMap<String, String>> {
    // Parse as dotenv lines
    let s = String::from_utf8_lossy(bytes);
    let mut map = BTreeMap::new();
    for line in s.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') { continue; }
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

/// Resolve a set directory from id or name; supports "global".
fn resolve_set_dir(id_or_name: &str) -> Result<PathBuf> {
    let cfg = config::load_config()?;
    if id_or_name == "global" { return config::global_dir(); }
    // accept name or id
    if let Some(s) = cfg.sets.iter().find(|s| s.id == id_or_name || s.name == id_or_name) {
        return config::set_dir(&s.id);
    }
    bail!("set not found: {}", id_or_name)
}

/// Load encryption key for dir (password-derived if locked, else app key). Uses SAFEHOLD_PASSWORD if set.
fn load_key_for_dir(dir: &PathBuf) -> Result<[u8;32]> {
    let base = config::base_dir()?;
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

/// Decrypt and read env map from a set directory.
fn read_env_map(dir: &PathBuf) -> Result<BTreeMap<String, String>> {
    let key = load_key_for_dir(dir)?;
    let enc = fs::read(env_enc_path(dir)) .unwrap_or_default();
    if enc.is_empty() { return Ok(BTreeMap::new()); }
    let pt = crypto::decrypt_with_key(&key, &enc)?;
    read_env_map_from_bytes(&pt)
}

/// Encrypt and write env map to a set directory.
fn write_env_map(dir: &PathBuf, map: &BTreeMap<String, String>) -> Result<()> {
    let key = load_key_for_dir(dir)?;
    let s = write_env_string(map);
    let ct = crypto::encrypt_with_key(&key, s.as_bytes())?;
    fs::write(env_enc_path(dir), ct)?;
    Ok(())
}

/// Add or replace a key/value in a set. Reads value from stdin if not provided.
pub fn cmd_add(args: SetKeyValueArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.set)?;
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
pub fn cmd_get(args: SetKeyArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.set)?;
    let map = read_env_map(&dir)?;
    if let Some(v) = map.get(&args.key) { println!("{}", v); } else { bail!("key not found"); }
    Ok(())
}

/// List all key=value pairs in a set.
pub fn cmd_list(args: SetTargetArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.set)?;
    let map = read_env_map(&dir)?;
    for (k, v) in map { println!("{}={}", k, v); }
    Ok(())
}

/// Delete a key in a set (no-op if missing).
pub fn cmd_delete(args: SetKeyArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.set)?;
    let mut map = read_env_map(&dir)?;
    if map.remove(&args.key).is_some() {
        write_env_map(&dir, &map)?;
        styles::ok("Deleted");
    } else {
        styles::warn("Key not found");
    }
    Ok(())
}

/// Export a set or global into a .env file; supports temp mode and overwrite.
pub fn cmd_export(args: ExportArgs) -> Result<()> {
    let dir = if args.global { config::global_dir()? } else {
        let set = args.set.as_deref().ok_or_else(|| anyhow::anyhow!("--set or --global required"))?;
        resolve_set_dir(set)?
    };
    let pb = styles::spinner("Decrypting and writing .env...");
    let map = read_env_map(&dir)?;
    let filename = args.file.unwrap_or_else(|| ".env".into());
    if std::path::Path::new(&filename).exists() && !args.force { bail!("{} exists, use --force", filename); }
    let content = write_env_string(&map);
    fs::write(&filename, content)?;
    if args.temp {
        // best-effort delete on exit
        let name = filename.clone();
        ctrlc::set_handler(move || { let _ = std::fs::remove_file(&name); std::process::exit(0); }).ok();
        styles::info(format!("Temporary .env written: {} (will delete on Ctrl+C or process exit)", filename));
    } else {
        styles::ok(format!(".env written: {}", filename));
    }
    styles::finish_spinner(pb, "Done");
    Ok(())
}

/// Run a program with environment variables injected from a set and optionally from global.
pub fn cmd_run(args: RunArgs) -> Result<()> {
    let dir = resolve_set_dir(&args.set)?;
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
    if !status.success() { bail!("process exited with status {:?}", status.code()); }
    Ok(())
}

/// Show all sets and their keys to stdout.
pub fn cmd_show_all() -> Result<()> {
    let cfg = config::load_config()?;
    styles::info("GLOBAL:");
    let gdir = config::global_dir()?;
    if let Ok(map) = read_env_map(&gdir) { for (k,v) in map { println!("  {}={}", k, v); } }
    for s in cfg.sets {
        styles::info(format!("SET {} ({})", s.id, s.name));
        let dir = config::set_dir(&s.id)?;
        if let Ok(map) = read_env_map(&dir) { for (k,v) in map { println!("  {}={}", k, v); } }
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
    styles::ok(format!("Removed {} .env files", removed));
    Ok(())
}
