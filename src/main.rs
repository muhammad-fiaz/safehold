//! SafeHold - Secure Cross-Platform Credential Manager
//!
//! SafeHold is a professional-grade credential manager that provides secure,
//! encrypted storage for environment variables and secrets with both CLI and
//! optional GUI interfaces.

mod cli;
mod core;
mod gui;
mod operations;
mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Fallback plain error (styles may or may not be initialized)
        eprintln!("safehold error: {e}");
        std::process::exit(1);
    }
}

/// Build CLI, initialize styles, and execute command.
async fn run() -> Result<()> {
    let cmd = cli::cli::build_cli();

    // Check version compatibility and show upgrade messages
    if let Ok((is_new, old_version)) = core::config::check_version_compatibility() {
        if !is_new {
            if let Some(old_version) = old_version {
                // Version upgrade detected
                core::config::display_version_message(&old_version)?;
            }
        }
    }

    // Check for updates on every command execution (non-blocking)
    if !matches!(cmd.command, cli::cli::Commands::CheckUpdate) {
        check_for_updates_background().await;
    }

    // Handle GUI flag for installation
    if cmd.gui {
        // Skip installation prompt if --gui flag is used
        println!("🖥️ Installing SafeHold with GUI support...");
        let _install_config = utils::install::InstallConfig {
            gui_enabled: true,
            skip_setup: false,
        };
    } else if should_run_install_prompt(&cmd) {
        // Handle installation prompt for first-time users
        let _install_config = utils::install::run_install_prompt()?;
    }

    cli::cli::dispatch(cmd).await
}

/// Determine if we should run the installation prompt
fn should_run_install_prompt(cmd: &cli::cli::Cli) -> bool {
    // Don't run install prompt for help commands, setup, or if explicitly skipped
    match &cmd.command {
        cli::cli::Commands::Setup { .. } => false, // Setup is manual
        _ => utils::install::is_first_run(),       // Only run on first run
    }
}

/// Background check for updates without blocking CLI execution
async fn check_for_updates_background() {
    tokio::spawn(async {
        if let Ok(Some(update_info)) = utils::update_checker::check_for_updates().await {
            utils::update_checker::display_update_notification(&update_info);
        }
    });
}
