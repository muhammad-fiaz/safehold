//! Application settings management for SafeHold
//!
//! This module handles persistent settings for both CLI and GUI modes,
//! storing user preferences separately from project configurations.

use crate::core::config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings that persist across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// GUI-specific settings
    pub gui: GuiSettings,
    /// CLI-specific settings  
    pub cli: CliSettings,
    /// Security settings
    pub security: SecuritySettings,
    /// Version when settings were last updated
    pub settings_version: String,
}

/// GUI interface settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiSettings {
    /// Show password values by default in GUI
    pub show_passwords_default: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    /// Window width (if we want to remember window size)
    pub window_width: f32,
    /// Window height
    pub window_height: f32,
    /// Remember window position
    pub remember_window_position: bool,
    /// Default tab to open
    pub default_tab: String,
}

/// CLI interface settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSettings {
    /// Default color output preference
    pub default_color: String, // "auto", "always", "never"
    /// Default style preference
    pub default_style: String, // "fancy", "plain"
    /// Show detailed help by default
    pub verbose_help: bool,
    /// Confirm destructive operations by default
    pub confirm_destructive: bool,
}

/// Security-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Global master lock enabled - when true, ALL projects use same global password
    pub global_master_lock: bool,
    /// Session timeout in minutes (0 = no timeout)
    pub session_timeout_minutes: u32,
    /// Clear clipboard after copying credentials (seconds)
    pub clipboard_clear_seconds: u32,
    /// Require confirmation for all destructive operations
    pub require_confirmation: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            gui: GuiSettings {
                show_passwords_default: false,
                auto_save_interval: 30,
                window_width: 1200.0,
                window_height: 800.0,
                remember_window_position: false,
                default_tab: "Projects".to_string(),
            },
            cli: CliSettings {
                default_color: "auto".to_string(),
                default_style: "fancy".to_string(),
                verbose_help: false,
                confirm_destructive: true,
            },
            security: SecuritySettings {
                global_master_lock: false,  // Default is unlocked
                session_timeout_minutes: 0, // No timeout by default
                clipboard_clear_seconds: 30,
                require_confirmation: true,
            },
            settings_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Get the path to the application settings file
pub fn settings_path() -> Result<PathBuf> {
    Ok(config::base_dir()?.join("app_settings.json"))
}

/// Load application settings from disk, creating defaults if not found
pub fn load_settings() -> Result<AppSettings> {
    let path = settings_path()?;

    if !path.exists() {
        // Create default settings file
        let settings = AppSettings::default();
        save_settings(&settings)?;
        return Ok(settings);
    }

    let data = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    let settings: AppSettings =
        serde_json::from_slice(&data).with_context(|| format!("parse {}", path.display()))?;

    Ok(settings)
}

/// Save application settings to disk
pub fn save_settings(settings: &AppSettings) -> Result<()> {
    let path = settings_path()?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }

    let data = serde_json::to_vec_pretty(settings)?;
    fs::write(&path, data).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Update GUI settings and save
#[allow(dead_code)]
pub fn update_gui_settings<F>(updater: F) -> Result<()>
where
    F: FnOnce(&mut GuiSettings),
{
    let mut settings = load_settings()?;
    updater(&mut settings.gui);
    save_settings(&settings)
}

/// Update CLI settings and save
#[allow(dead_code)]
pub fn update_cli_settings<F>(updater: F) -> Result<()>
where
    F: FnOnce(&mut CliSettings),
{
    let mut settings = load_settings()?;
    updater(&mut settings.cli);
    save_settings(&settings)
}

/// Update security settings and save
pub fn update_security_settings<F>(updater: F) -> Result<()>
where
    F: FnOnce(&mut SecuritySettings),
{
    let mut settings = load_settings()?;
    updater(&mut settings.security);
    save_settings(&settings)
}

/// Check if global master lock is enabled
#[allow(dead_code)]
pub fn is_global_master_lock_enabled() -> bool {
    load_settings()
        .map(|s| s.security.global_master_lock)
        .unwrap_or(false)
}

/// Enable or disable global master lock
pub fn set_global_master_lock(enabled: bool) -> Result<()> {
    update_security_settings(|security| {
        security.global_master_lock = enabled;
    })
}

/// Get CLI defaults for color and style
#[allow(dead_code)]
pub fn get_cli_defaults() -> (String, String) {
    load_settings()
        .map(|s| (s.cli.default_color, s.cli.default_style))
        .unwrap_or_else(|_| ("auto".to_string(), "fancy".to_string()))
}
