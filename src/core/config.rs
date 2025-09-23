//! Configuration and filesystem layout helpers for SafeHold.
use anyhow::{Context, Result};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

/// Application base directory under the user's home.
pub const APP_DIR_NAME: &str = ".safehold";

/// Persistent configuration stored in `config.json` under the base dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sets: Vec<SetMeta>,
    pub global_locked: bool,
    pub created_at: String,
    #[serde(default = "default_version")]
    pub version: String,
}

/// Metadata for each credential project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetMeta {
    pub id: String,   // e.g. 001_project1
    pub name: String, // display name
    pub locked: bool, // requires password
}

fn default_version() -> String {
    "0.0.2".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sets: vec![],
            global_locked: false,
            created_at: OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            version: default_version(),
        }
    }
}

pub fn home_dir() -> Result<PathBuf> {
    let dirs = UserDirs::new().context("No home directory found")?;
    Ok(dirs.home_dir().to_path_buf())
}

pub fn base_dir() -> Result<PathBuf> {
    if let Ok(special) = std::env::var("SAFEHOLD_HOME") {
        return Ok(PathBuf::from(special));
    }
    Ok(home_dir()?.join(APP_DIR_NAME))
}

pub fn ensure_layout() -> Result<PathBuf> {
    let base = base_dir()?;
    let cred = base.join("credentials");
    let global = base.join("global");
    fs::create_dir_all(&cred).with_context(|| format!("create {}", cred.display()))?;
    fs::create_dir_all(&global).with_context(|| format!("create {}", global.display()))?;
    let cfg_path = base.join("config.json");
    if !cfg_path.exists() {
        let cfg = Config::default();
        fs::write(&cfg_path, serde_json::to_vec_pretty(&cfg)?)?;
    }
    Ok(base)
}

pub fn load_config() -> Result<Config> {
    let path = base_dir()?.join("config.json");
    let data = fs::read(&path).with_context(|| format!("read {}", path.display()))?;

    // Try to parse as current version first
    match serde_json::from_slice::<Config>(&data) {
        Ok(mut cfg) => {
            // If version is missing or old, update it
            if cfg.version.is_empty() || cfg.version == "0.0.1" {
                cfg.version = default_version();
                // Save the updated config to migrate it
                save_config(&cfg)?;
            }
            Ok(cfg)
        }
        Err(_) => {
            // Try to parse as legacy config (without version field)
            #[derive(Deserialize)]
            struct LegacyConfig {
                pub sets: Vec<SetMeta>,
                pub global_locked: bool,
                pub created_at: String,
            }

            let legacy: LegacyConfig = serde_json::from_slice(&data)
                .with_context(|| format!("parse legacy config {}", path.display()))?;

            // Migrate to new format
            let cfg = Config {
                sets: legacy.sets,
                global_locked: legacy.global_locked,
                created_at: legacy.created_at,
                version: default_version(),
            };

            // Save the migrated config
            save_config(&cfg)?;
            Ok(cfg)
        }
    }
}

pub fn save_config(cfg: &Config) -> Result<()> {
    let path = base_dir()?.join("config.json");
    let data = serde_json::to_vec_pretty(cfg)?;
    fs::write(&path, data).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn set_dir(id: &str) -> Result<PathBuf> {
    Ok(base_dir()?.join("credentials").join(id))
}
pub fn global_dir() -> Result<PathBuf> {
    Ok(base_dir()?.join("global"))
}

pub fn next_set_id(name: &str, existing: &[SetMeta]) -> String {
    let mut maxn = 0u32;
    for s in existing {
        if let Some((n, _)) = s.id.split_once('_') {
            if let Ok(v) = n.parse::<u32>() {
                if v > maxn {
                    maxn = v;
                }
            }
        }
    }
    format!("{:03}_{}", maxn + 1, name)
}

pub fn lock_path(dir: &Path) -> PathBuf {
    dir.join("lock.json")
}
pub fn env_enc_path(dir: &Path) -> PathBuf {
    dir.join(".env.enc")
}

/// Version tracking for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub last_updated: String,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_updated: OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        }
    }
}

/// Get the version file path
pub fn version_file_path() -> Result<PathBuf> {
    Ok(base_dir()?.join("version.json"))
}

/// Check if this is a new installation or upgrade
pub fn check_version_compatibility() -> Result<(bool, Option<String>)> {
    let version_path = version_file_path()?;
    let current_version = env!("CARGO_PKG_VERSION");

    if !version_path.exists() {
        // New installation
        save_current_version()?;
        return Ok((true, None));
    }

    // Read existing version
    let version_content = fs::read_to_string(&version_path)?;
    let version_info: VersionInfo =
        serde_json::from_str(&version_content).unwrap_or_else(|_| VersionInfo::default());

    if version_info.version != current_version {
        // Version changed - save new version and return old version
        save_current_version()?;
        return Ok((false, Some(version_info.version)));
    }

    // Same version
    Ok((false, None))
}

/// Save the current version to the version file
pub fn save_current_version() -> Result<()> {
    let version_path = version_file_path()?;
    let version_info = VersionInfo::default();

    // Ensure directory exists
    if let Some(parent) = version_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(&version_info)?;
    fs::write(version_path, content)?;
    Ok(())
}

/// Display version compatibility message
pub fn display_version_message(old_version: &str) -> Result<()> {
    use crate::cli::styles;

    println!();
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│                    🔄 SafeHold Version Update                    │");
    println!("│                                                                 │");
    println!(
        "│  Previous Version: {}                                        │",
        format!("{:<10}", old_version)
    );
    println!(
        "│  Current Version:  {}                                        │",
        format!("{:<10}", env!("CARGO_PKG_VERSION"))
    );
    println!("│                                                                 │");
    println!("│  ✅ Your data has been preserved and is compatible             │");
    println!("│  ✅ All existing functionality remains unchanged               │");
    println!("│  ✅ New features are now available!                            │");
    println!("│                                                                 │");
    println!("│  📖 Run 'safehold --help' to see new commands                  │");
    println!("│  🌍 Try 'safehold global-list' for global credentials         │");
    println!("│  📊 Use 'safehold count' for credential statistics             │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    styles::success(&format!(
        "🎉 Update completed successfully! Welcome to SafeHold v{}",
        env!("CARGO_PKG_VERSION")
    ));

    Ok(())
}
