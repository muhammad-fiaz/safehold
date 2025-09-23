mod app_settings;
mod cli;
mod config;
mod crypto;
mod envops;
mod install;
mod master_lock;
mod store;
mod styles;
#[cfg(feature = "gui")]
mod ui;

use anyhow::Result;

fn main() {
    if let Err(e) = run() {
        // Fallback plain error (styles may or may not be initialized)
        eprintln!("safehold error: {e}");
        std::process::exit(1);
    }
}

/// Build CLI, initialize styles, and execute command.
fn run() -> Result<()> {
    let cmd = cli::build_cli();
    
    // Check version compatibility and show upgrade messages
    if let Ok((is_new, old_version)) = config::check_version_compatibility() {
        if !is_new {
            if let Some(old_version) = old_version {
                // Version upgrade detected
                config::display_version_message(&old_version)?;
            }
        }
    }
    
    // Handle GUI flag for installation
    if cmd.gui {
        // Skip installation prompt if --gui flag is used
        println!("🖥️ Installing SafeHold with GUI support...");
        let _install_config = install::InstallConfig {
            gui_enabled: true,
            skip_setup: false,
        };
    } else if should_run_install_prompt(&cmd) {
        // Handle installation prompt for first-time users
        let _install_config = install::run_install_prompt()?;
    }
    
    cli::dispatch(cmd)
}

/// Determine if we should run the installation prompt
fn should_run_install_prompt(cmd: &cli::Cli) -> bool {
    // Don't run install prompt for help commands, setup, or if explicitly skipped
    match &cmd.command {
        cli::Commands::Setup { .. } => false, // Setup is manual
        _ => install::is_first_run(), // Only run on first run
    }
}
