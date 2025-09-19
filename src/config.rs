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
}

/// Metadata for each credential project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetMeta {
    pub id: String,    // e.g. 001_project1
    pub name: String,  // display name
    pub locked: bool,  // requires password
}

impl Default for Config {
    fn default() -> Self {
        Self { sets: vec![], global_locked: false, created_at: OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or_default() }
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
    let cfg = serde_json::from_slice(&data).with_context(|| format!("parse {}", path.display()))?;
    Ok(cfg)
}

pub fn save_config(cfg: &Config) -> Result<()> {
    let path = base_dir()?.join("config.json");
    let data = serde_json::to_vec_pretty(cfg)?;
    fs::write(&path, data).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn set_dir(id: &str) -> Result<PathBuf> { Ok(base_dir()?.join("credentials").join(id)) }
pub fn global_dir() -> Result<PathBuf> { Ok(base_dir()?.join("global")) }

pub fn next_set_id(name: &str, existing: &[SetMeta]) -> String {
    let mut maxn = 0u32;
    for s in existing {
        if let Some((n, _)) = s.id.split_once('_') {
            if let Ok(v) = n.parse::<u32>() { if v > maxn { maxn = v; } }
        }
    }
    format!("{:03}_{}", maxn + 1, name)
}

pub fn lock_path(dir: &Path) -> PathBuf { dir.join("lock.json") }
pub fn env_enc_path(dir: &Path) -> PathBuf { dir.join(".env.enc") }