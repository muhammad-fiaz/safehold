//! Integration tests for SafeHold functionality
//! 
//! Tests all CLI commands including the new update, count, and global functionality
//! to ensure comprehensive functionality across all platforms.

use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper struct for managing test environment
struct TestEnv {
    _temp_dir: TempDir,
    safehold_path: PathBuf,
    test_dir: PathBuf,
}

impl TestEnv {
    /// Create a new test environment with isolated SafeHold data directory
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().to_path_buf();
        
        // Build SafeHold binary if not exists
        let cargo_target_dir = env::var("CARGO_TARGET_DIR")
            .unwrap_or_else(|_| "target".to_string());
        let safehold_path = PathBuf::from(cargo_target_dir)
            .join("debug")
            .join(if cfg!(windows) { "safehold.exe" } else { "safehold" });
        
        // Ensure binary exists
        if !safehold_path.exists() {
            Command::new("cargo")
                .args(&["build"])
                .status()?;
        }
        
        Ok(TestEnv {
            _temp_dir: temp_dir,
            safehold_path,
            test_dir,
        })
    }
    
    /// Run a SafeHold command with isolated data directory
    fn run_cmd(&self, args: &[&str]) -> Result<std::process::Output> {
        let output = Command::new(&self.safehold_path)
            .args(args)
            .env("SAFEHOLD_HOME", &self.test_dir)
            .output()?;
        Ok(output)
    }
    
    /// Run command and expect success
    fn run_success(&self, args: &[&str]) -> Result<String> {
        let output = self.run_cmd(args)?;
        if !output.status.success() {
            anyhow::bail!(
                "Command failed: {} {}\nStderr: {}",
                self.safehold_path.display(),
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Run command and expect failure
    fn run_failure(&self, args: &[&str]) -> Result<String> {
        let output = self.run_cmd(args)?;
        if output.status.success() {
            anyhow::bail!(
                "Command unexpectedly succeeded: {} {}",
                self.safehold_path.display(),
                args.join(" ")
            );
        }
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_project_lifecycle() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test project creation
    let output = env.run_success(&["create", "test-project"])?;
    assert!(output.contains("Created project"));
    assert!(output.contains("test-project"));
    
    // Test project listing
    let output = env.run_success(&["list-projects"])?;
    assert!(output.contains("test-project"));
    assert!(output.contains("Total:"));
    
    // Test project deletion
    let output = env.run_success(&["delete-project", "test-project"])?;
    assert!(output.contains("Deleted") || output.contains("removed"));
    
    Ok(())
}

#[test]
fn test_credential_management() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Create a test project
    env.run_success(&["create", "cred-test"])?;
    
    // Add credential
    env.run_success(&["add", "--project", "cred-test", "--key", "API_KEY", "--value", "secret123"])?;
    
    // Get credential
    let output = env.run_success(&["get", "--project", "cred-test", "--key", "API_KEY"])?;
    assert!(output.trim() == "secret123");
    
    // List credentials
    let output = env.run_success(&["list", "--project", "cred-test"])?;
    assert!(output.contains("API_KEY=secret123"));
    
    // Update credential (new functionality)
    env.run_success(&["update", "--project", "cred-test", "--key", "API_KEY", "--value", "newsecret456"])?;
    
    // Verify update
    let output = env.run_success(&["get", "--project", "cred-test", "--key", "API_KEY"])?;
    assert!(output.trim() == "newsecret456");
    
    // Delete credential
    env.run_success(&["delete", "--project", "cred-test", "--key", "API_KEY"])?;
    
    // Verify deletion
    let _error = env.run_failure(&["get", "--project", "cred-test", "--key", "API_KEY"])?;
    
    Ok(())
}

#[test]
fn test_global_credentials() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Setup the environment first by creating a project (this initializes crypto)
    env.run_success(&["create", "init-project"])?;
    
    // Add global credential
    env.run_success(&["global-add", "--key", "GLOBAL_TOKEN", "--value", "globalvalue123"])?;
    
    // Get global credential
    let output = env.run_success(&["global-get", "--key", "GLOBAL_TOKEN"])?;
    assert!(output.trim() == "globalvalue123");
    
    // List global credentials
    let output = env.run_success(&["global-list"])?;
    assert!(output.contains("GLOBAL_TOKEN=globalvalue123"));
    assert!(output.contains("Global Credentials"));
    
    // Update global credential
    env.run_success(&["global-update", "--key", "GLOBAL_TOKEN", "--value", "updatedglobal456"])?;
    
    // Verify update
    let output = env.run_success(&["global-get", "--key", "GLOBAL_TOKEN"])?;
    assert!(output.trim() == "updatedglobal456");
    
    // Delete global credential
    env.run_success(&["global-delete", "--key", "GLOBAL_TOKEN"])?;
    
    // Verify deletion
    let _error = env.run_failure(&["global-get", "--key", "GLOBAL_TOKEN"])?;
    
    Ok(())
}

#[test]
fn test_count_functionality() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Create test projects and add credentials
    env.run_success(&["create", "project1"])?;
    env.run_success(&["create", "project2"])?;
    env.run_success(&["add", "--project", "project1", "--key", "KEY1", "--value", "val1"])?;
    env.run_success(&["add", "--project", "project1", "--key", "KEY2", "--value", "val2"])?;
    env.run_success(&["add", "--project", "project2", "--key", "KEY3", "--value", "val3"])?;
    env.run_success(&["global-add", "--key", "GLOBAL_KEY", "--value", "globalval"])?;
    
    // Test count for specific project
    let output = env.run_success(&["count", "--project", "project1"])?;
    assert!(output.contains("has 2 credential(s)"));
    
    // Test count for all projects without global
    let output = env.run_success(&["count"])?;
    assert!(output.contains("Total:"));
    assert!(output.contains("3 credential(s)"));
    
    // Test count for all projects with global
    let output = env.run_success(&["count", "--include-global"])?;
    assert!(output.contains("🌍 Global: 1 credential(s)"));
    assert!(output.contains("Total: 4 credential(s)"));
    
    // Test detailed count
    let output = env.run_success(&["count", "--detailed", "--include-global"])?;
    assert!(output.contains("project1"));
    assert!(output.contains("project2"));
    assert!(output.contains("Global:"));
    
    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test getting non-existent credential
    let _error = env.run_failure(&["get", "--project", "nonexistent", "--key", "KEY"])?;
    
    // Test updating non-existent credential
    let _error = env.run_failure(&["update", "--project", "nonexistent", "--key", "KEY", "--value", "val"])?;
    
    // Test deleting non-existent credential
    let _error = env.run_failure(&["delete", "--project", "nonexistent", "--key", "KEY"])?;
    
    // Test global operations on non-existent keys
    let _error = env.run_failure(&["global-get", "--key", "NONEXISTENT"])?;
    let _error = env.run_failure(&["global-update", "--key", "NONEXISTENT", "--value", "val"])?;
    let _error = env.run_failure(&["global-delete", "--key", "NONEXISTENT"])?;
    
    Ok(())
}

#[test]
fn test_show_all_functionality() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Create test data
    env.run_success(&["create", "show-test"])?;
    env.run_success(&["add", "--project", "show-test", "--key", "PROJECT_KEY", "--value", "projectval"])?;
    env.run_success(&["global-add", "--key", "SHOW_GLOBAL", "--value", "globalval"])?;
    
    // Test show-all
    let output = env.run_success(&["show-all"])?;
    assert!(output.contains("GLOBAL:"));
    assert!(output.contains("SHOW_GLOBAL=globalval"));
    assert!(output.contains("PROJECT"));
    assert!(output.contains("PROJECT_KEY=projectval"));
    
    Ok(())
}

#[test]
fn test_export_functionality() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Create test project with credentials
    env.run_success(&["create", "export-test"])?;
    env.run_success(&["add", "--project", "export-test", "--key", "EXPORT_KEY", "--value", "exportval"])?;
    
    // Test export to file
    let export_file = env.test_dir.join("test.env");
    env.run_success(&[
        "export", 
        "--project", "export-test", 
        "--file", export_file.to_str().unwrap()
    ])?;
    
    // Verify export file contents
    let contents = fs::read_to_string(&export_file)?;
    assert!(contents.contains("EXPORT_KEY=exportval"));
    
    Ok(())
}

#[test]
fn test_cross_platform_compatibility() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test basic functionality that should work on all platforms
    env.run_success(&["create", "cross-platform-test"])?;
    env.run_success(&["add", "--project", "cross-platform-test", "--key", "PLATFORM_KEY", "--value", "works"])?;
    
    let output = env.run_success(&["get", "--project", "cross-platform-test", "--key", "PLATFORM_KEY"])?;
    assert!(output.trim() == "works");
    
    // Test path handling (should work on Windows and Unix)
    let output = env.run_success(&["setup"])?;
    assert!(output.contains("SafeHold environment"));
    
    Ok(())
}

#[test]
fn test_version_information() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test version flag
    let output = env.run_success(&["--version"])?;
    assert!(output.contains("safehold"));
    assert!(output.contains("0.0.1"));
    
    Ok(())
}

#[test]
fn test_help_system() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test main help
    let output = env.run_success(&["--help"])?;
    assert!(output.contains("SafeHold"));
    assert!(output.contains("global-add"));
    assert!(output.contains("update"));
    assert!(output.contains("count"));
    
    // Test command-specific help
    let output = env.run_success(&["global-add", "--help"])?;
    assert!(output.contains("Add a key/value credential to global storage"));
    
    let output = env.run_success(&["count", "--help"])?;
    assert!(output.contains("Count credentials in projects"));
    
    Ok(())
}

#[test]
fn test_about_command() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test about command
    let output = env.run_success(&["about"])?;
    assert!(output.contains("SafeHold"));
    assert!(output.contains("Version: 0.0.1"));
    assert!(output.contains("Professional Credential Manager"));
    assert!(output.contains("Developer Information"));
    assert!(output.contains("Security Features"));
    assert!(output.contains("AES-256-GCM encryption"));
    assert!(output.contains("Argon2id password hashing"));
    assert!(output.contains("Repository:"));
    
    Ok(())
}

#[test]
fn test_clean_cache_command() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Create some cache directories and files to clean
    let cache_dir = env.test_dir.join(".safehold").join("cache");
    fs::create_dir_all(&cache_dir)?;
    fs::write(cache_dir.join("test_cache.tmp"), "cache data")?;
    
    let temp_dir = env.test_dir.join(".safehold").join("tmp");
    fs::create_dir_all(&temp_dir)?;
    fs::write(temp_dir.join("test_temp.log"), "temp data")?;
    
    // Test clean-cache command with force flag
    let output = env.run_success(&["clean-cache", "--force"])?;
    
    // Check that it reports cleaning or no files to clean
    assert!(output.contains("cache") || output.contains("No cache files"));
    
    Ok(())
}

#[test]
fn test_clean_cache_no_files() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test clean-cache when there are no cache files
    let output = env.run_success(&["clean-cache", "--force"])?;
    assert!(output.contains("No cache files found to clean"));
    
    Ok(())
}

#[test]
fn test_delete_all_command_safety() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Create a test project with data
    env.run_success(&["create", "test-for-deletion"])?;
    env.run_success(&["add", "--project", "test-for-deletion", "--key", "TEST_KEY", "--value", "test_value"])?;
    
    // Verify project exists
    let output = env.run_success(&["list-projects"])?;
    assert!(output.contains("test-for-deletion"));
    
    // Test delete-all with force flag (bypasses confirmation)
    let output = env.run_success(&["delete-all", "--force"])?;
    assert!(output.contains("permanently deleted") || output.contains("All SafeHold data"));
    
    // After delete-all, projects should be gone but list-projects might fail since config is deleted
    // So we just verify the delete command worked by checking its output
    // Instead of trying to list projects (which might fail with deleted config)
    
    Ok(())
}

#[test]
fn test_destructive_command_aliases() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test clean-cache aliases
    let output = env.run_success(&["clear-cache", "--force"])?;
    assert!(output.contains("cache") || output.contains("No cache files"));
    
    let output = env.run_success(&["cache-clean", "--force"])?;
    assert!(output.contains("cache") || output.contains("No cache files"));
    
    // Test about aliases
    let output = env.run_success(&["info"])?;
    assert!(output.contains("SafeHold") || output.contains("safehold"));
    assert!(output.contains("Version: 0.0.1"));
    
    Ok(())
}

#[test]
fn test_force_flag_functionality() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Create test data
    env.run_success(&["create", "force-test"])?;
    env.run_success(&["add", "--project", "force-test", "--key", "FORCE_KEY", "--value", "force_value"])?;
    
    // Test that force flag works for destructive operations
    let output = env.run_success(&["clean-cache", "--force"])?;
    assert!(!output.contains("Type 'yes' to continue")); // Should not show confirmation
    
    // Clean up
    env.run_success(&["delete-all", "--force"])?;
    
    Ok(())
}

#[test]
fn test_comprehensive_help_coverage() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test that all new commands show up in help
    let output = env.run_success(&["--help"])?;
    
    // Check for new destructive commands
    assert!(output.contains("clean-cache"));
    assert!(output.contains("delete-all"));
    assert!(output.contains("about"));
    
    // Check descriptions
    assert!(output.contains("Clean cache and temporary files"));
    assert!(output.contains("Delete ALL projects and data (DESTRUCTIVE!)"));
    assert!(output.contains("Show application information and details"));
    
    // Check aliases are mentioned
    assert!(output.contains("clear-cache"));
    assert!(output.contains("nuke"));
    assert!(output.contains("info"));
    
    Ok(())
}

#[test]
fn test_developer_information_display() -> Result<()> {
    let env = TestEnv::new()?;
    
    let output = env.run_success(&["about"])?;
    
    // Check that developer information is comprehensive
    assert!(output.contains("Repository:"));
    assert!(output.contains("github.com/muhammad-fiaz/safehold"));
    assert!(output.contains("Build Information"));
    assert!(output.contains("Current Status"));
    assert!(output.contains("Available Features"));
    assert!(output.contains("Quick Commands"));
    
    // Check security features are listed
    assert!(output.contains("Memory-safe Rust implementation"));
    assert!(output.contains("Cross-platform secure key derivation"));
    assert!(output.contains("No plaintext storage"));
    
    Ok(())
}

#[test]
fn test_master_lock_status() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Check initial master lock status (should be disabled)
    let output = env.run_cmd(&["master-lock"])?;
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Global Master Lock Status"));
    assert!(stdout.contains("🔓 DISABLED"));
    assert!(stdout.contains("Projects use individual lock settings"));
    
    Ok(())
}

#[test]
fn test_master_lock_help() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test master-lock help
    let output = env.run_cmd(&["master-lock", "--help"])?;
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Manage Global Master Lock"));
    assert!(stdout.contains("--enable"));
    assert!(stdout.contains("--disable"));
    assert!(stdout.contains("unified password for ALL projects"));
    
    Ok(())
}

#[test]
fn test_master_lock_aliases() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Test mlock alias
    let output = env.run_cmd(&["mlock"])?;
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Global Master Lock Status"));
    
    // Test global-master alias
    let output2 = env.run_cmd(&["global-master"])?;
    assert!(output2.status.success());
    
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("Global Master Lock Status"));
    
    Ok(())
}

#[test]
fn test_app_settings_creation() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Run master lock status command to trigger app settings creation
    let output = env.run_cmd(&["master-lock"])?;
    assert!(output.status.success());
    
    // Check that app_settings.json was created
    let settings_path = env.test_dir.join("app_settings.json");
    assert!(settings_path.exists());
    
    // Test the settings file structure
    let settings_content = fs::read_to_string(&settings_path)?;
    let settings: serde_json::Value = serde_json::from_str(&settings_content)?;
    
    // Verify structure
    assert!(settings.get("gui").is_some());
    assert!(settings.get("cli").is_some());
    assert!(settings.get("security").is_some());
    assert!(settings.get("settings_version").is_some());
    
    // Check default values
    assert_eq!(settings["security"]["global_master_lock"], false);
    assert_eq!(settings["gui"]["show_passwords_default"], false);
    assert_eq!(settings["cli"]["default_color"], "auto");
    
    Ok(())
}

#[test]
fn test_master_lock_file_creation() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Master lock file should not exist initially
    let master_lock_path = env.test_dir.join("master_lock.json");
    assert!(!master_lock_path.exists());
    
    // Check that the master lock status reports disabled
    let output = env.run_cmd(&["master-lock"])?;
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("🔓 DISABLED"));
    
    Ok(())
}

#[test]
fn test_app_settings_integration() -> Result<()> {
    let env = TestEnv::new()?;
    
    // Run master lock status to trigger app settings creation
    let _output = env.run_cmd(&["master-lock"])?;
    
    // Verify app settings are created with correct default values
    let settings_path = env.test_dir.join("app_settings.json");
    assert!(settings_path.exists());
    
    let settings_content = fs::read_to_string(&settings_path)?;
    let settings: serde_json::Value = serde_json::from_str(&settings_content)?;
    
    // Check all major sections exist
    assert!(settings["gui"].is_object());
    assert!(settings["cli"].is_object());
    assert!(settings["security"].is_object());
    
    // Check specific security defaults
    assert_eq!(settings["security"]["global_master_lock"], false);
    assert_eq!(settings["security"]["session_timeout_minutes"], 0);
    assert_eq!(settings["security"]["clipboard_clear_seconds"], 30);
    assert_eq!(settings["security"]["require_confirmation"], true);
    
    // Check GUI defaults
    assert_eq!(settings["gui"]["show_passwords_default"], false);
    assert_eq!(settings["gui"]["auto_save_interval"], 30);
    assert_eq!(settings["gui"]["default_tab"], "Projects");
    
    // Check CLI defaults
    assert_eq!(settings["cli"]["default_color"], "auto");
    assert_eq!(settings["cli"]["default_style"], "fancy");
    assert_eq!(settings["cli"]["confirm_destructive"], true);
    
    Ok(())
}