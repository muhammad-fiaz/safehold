//! Global Master Lock functionality for SafeHold
//!
//! When Global Master Lock is enabled, ALL projects (including individual projects and global)
//! require the same master password for access. This provides unified security across all credentials.

use crate::app_settings;
use crate::config;
use crate::crypto;
use crate::styles;
use anyhow::{Result, bail};
use rpassword;
use std::fs;

/// Master lock information stored separately
#[derive(Debug, Clone)]
pub struct MasterLockInfo {
    pub enabled: bool,
    pub password_hash: Option<String>,
}

/// Path to the master lock file
fn master_lock_path() -> Result<std::path::PathBuf> {
    Ok(config::base_dir()?.join("master_lock.json"))
}

/// Load master lock information
pub fn load_master_lock_info() -> Result<MasterLockInfo> {
    let path = master_lock_path()?;

    if !path.exists() {
        return Ok(MasterLockInfo {
            enabled: false,
            password_hash: None,
        });
    }

    let data = fs::read_to_string(&path)?;
    let json: serde_json::Value = serde_json::from_str(&data)?;

    Ok(MasterLockInfo {
        enabled: json
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        password_hash: json
            .get("password_hash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

/// Save master lock information
pub fn save_master_lock_info(info: &MasterLockInfo) -> Result<()> {
    let path = master_lock_path()?;

    let json = if info.enabled {
        serde_json::json!({
            "enabled": true,
            "password_hash": info.password_hash
        })
    } else {
        serde_json::json!({
            "enabled": false
        })
    };

    fs::write(&path, serde_json::to_string_pretty(&json)?)?;
    Ok(())
}

/// Check if global master lock is enabled
pub fn is_master_lock_enabled() -> bool {
    load_master_lock_info()
        .map(|info| info.enabled)
        .unwrap_or(false)
}

/// Enable global master lock with password
pub fn enable_master_lock(password: &str) -> Result<()> {
    // Create a hash from the password for verification
    let password_hash = crypto::argon2_hash(password.as_bytes())?;

    let info = MasterLockInfo {
        enabled: true,
        password_hash: Some(password_hash),
    };

    save_master_lock_info(&info)?;

    // Update app settings
    app_settings::set_global_master_lock(true)?;

    styles::success("🔒 Global Master Lock ENABLED");
    styles::info("All projects now require the master password for access");

    Ok(())
}

/// Disable global master lock
pub fn disable_master_lock() -> Result<()> {
    let info = MasterLockInfo {
        enabled: false,
        password_hash: None,
    };

    save_master_lock_info(&info)?;

    // Update app settings
    app_settings::set_global_master_lock(false)?;

    styles::success("🔓 Global Master Lock DISABLED");
    styles::info("Projects now use their individual lock settings");

    Ok(())
}

/// Verify master password
pub fn verify_master_password(password: &str) -> Result<bool> {
    let info = load_master_lock_info()?;

    if !info.enabled {
        return Ok(true); // No master lock, always valid
    }

    let stored_hash = info
        .password_hash
        .ok_or_else(|| anyhow::anyhow!("Master lock enabled but no password hash found"))?;

    Ok(crypto::argon2_verify(password.as_bytes(), &stored_hash)?)
}

/// Prompt for master password if master lock is enabled
pub fn prompt_master_password_if_needed() -> Result<Option<String>> {
    if !is_master_lock_enabled() {
        return Ok(None);
    }

    styles::info("🔒 Global Master Lock is enabled");

    loop {
        let password = rpassword::prompt_password("Enter master password: ")?;

        if verify_master_password(&password)? {
            styles::success("✅ Master password verified");
            return Ok(Some(password));
        } else {
            styles::error("❌ Invalid master password. Try again.");
        }
    }
}

/// Check if access is allowed to any project operations
/// Returns the master password if master lock is enabled and verified
#[allow(dead_code)]
pub fn check_master_access() -> Result<Option<String>> {
    prompt_master_password_if_needed()
}

/// Display master lock status
pub fn display_master_lock_status() -> Result<()> {
    let info = load_master_lock_info()?;

    // Initialize app settings if they don't exist
    let _app_settings = app_settings::load_settings()?;

    styles::header("Global Master Lock Status");
    styles::divider();

    if info.enabled {
        styles::kv("Status", "🔒 ENABLED");
        styles::kv("Protection", "ALL projects require master password");
        styles::kv("Security", "High - Unified password protection");
    } else {
        styles::kv("Status", "🔓 DISABLED");
        styles::kv("Protection", "Projects use individual lock settings");
        styles::kv("Security", "Standard - Per-project passwords");
    }

    println!();
    styles::info("Use 'safehold master-lock --enable' to enable global protection");
    styles::info("Use 'safehold master-lock --disable' to disable global protection");

    Ok(())
}

/// Master lock command handler
pub fn cmd_master_lock(enable: Option<bool>) -> Result<()> {
    match enable {
        Some(true) => {
            if is_master_lock_enabled() {
                styles::warn("Global Master Lock is already enabled");
                return Ok(());
            }

            styles::info("Setting up Global Master Lock...");
            styles::warn("⚠️  This will require a master password for ALL project access");

            let password = rpassword::prompt_password("Create master password: ")?;
            let confirm = rpassword::prompt_password("Confirm master password: ")?;

            if password != confirm {
                bail!("Passwords do not match");
            }

            if password.len() < 8 {
                bail!("Master password must be at least 8 characters long");
            }

            enable_master_lock(&password)?;
        }
        Some(false) => {
            if !is_master_lock_enabled() {
                styles::warn("Global Master Lock is already disabled");
                return Ok(());
            }

            // Verify current master password before disabling
            if let Some(_) = prompt_master_password_if_needed()? {
                disable_master_lock()?;
            } else {
                bail!("Master password verification required to disable");
            }
        }
        None => {
            display_master_lock_status()?;
        }
    }

    Ok(())
}
