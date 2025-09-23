use anyhow::Result;
use std::io::{self, Write};
use crate::styles;

/// Installation configuration options
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InstallConfig {
    pub gui_enabled: bool,
    pub skip_setup: bool,
}

/// Interactive installation prompt for first-time users
pub fn run_install_prompt() -> Result<InstallConfig> {
    // Check if this is a fresh installation
    if !is_first_run() {
        return Ok(InstallConfig {
            gui_enabled: cfg!(feature = "gui"),
            skip_setup: false,
        });
    }

    // Display ASCII art banner
    println!();
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│  ███████╗ █████╗ ███████╗███████╗██╗  ██╗ ██████╗ ██╗     ██████╗ │");
    println!("│  ██╔════╝██╔══██╗██╔════╝██╔════╝██║  ██║██╔═══██╗██║     ██╔══██╗│");
    println!("│  ███████╗███████║█████╗  █████╗  ███████║██║   ██║██║     ██║  ██║│");
    println!("│  ╚════██║██╔══██║██╔══╝  ██╔══╝  ██╔══██║██║   ██║██║     ██║  ██║│");
    println!("│  ███████║██║  ██║██║     ███████╗██║  ██║╚██████╔╝███████╗██████╔╝│");
    println!("│  ╚══════╝╚═╝  ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═════╝ │");
    println!("│                                                                     │");
    println!("│              🔐 Professional Environment Variable Manager 🔐       │");
    println!("│                      Version {} - Welcome! 🎉                  │", env!("CARGO_PKG_VERSION"));
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    styles::info("Welcome to SafeHold - Secure Credential Manager!");
    println!();
    
    styles::info("SafeHold can be used in two modes:");
    println!("  📝 CLI-only: Command-line interface (default, lightweight)");
    if cfg!(feature = "gui") {
        println!("  🖥️  GUI mode: Graphical user interface (optional)");
    }
    println!();

    let gui_enabled = if cfg!(feature = "gui") {
        prompt_gui_selection()?
    } else {
        false
    };

    // Create initial config to mark installation as complete
    mark_installation_complete()?;

    styles::success("✅ Installation configured successfully!");
    println!();

    if gui_enabled {
        styles::info("💡 To launch GUI: safehold launch --gui");
    }
    styles::info("💡 To see all commands: safehold --help");
    styles::info("💡 To add to PATH: safehold setup --add-path");
    println!();

    Ok(InstallConfig {
        gui_enabled,
        skip_setup: false,
    })
}

/// Prompt user to select between CLI-only and GUI modes
fn prompt_gui_selection() -> Result<bool> {
    loop {
        styles::info("Choose your preferred mode:");
        println!("  1) CLI-only (recommended for servers/headless)");
        println!("  2) GUI mode (recommended for desktop use)");
        print!("Enter choice (1-2) [default: 1]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "" | "1" => {
                styles::success("📝 CLI-only mode selected");
                return Ok(false);
            }
            "2" => {
                styles::success("🖥️ GUI mode selected");
                return Ok(true);
            }
            _ => {
                styles::warn("Invalid choice. Please enter 1 or 2.");
                continue;
            }
        }
    }
}

/// Check if this is the first run of SafeHold
pub fn is_first_run() -> bool {
    let config_dir = match crate::config::base_dir() {
        Ok(dir) => dir,
        Err(_) => return true,
    };

    // Check if config directory exists and has the installation marker
    let install_marker = config_dir.join(".installed");
    !install_marker.exists()
}

/// Mark installation as complete
fn mark_installation_complete() -> Result<()> {
    let config_dir = crate::config::base_dir()?;
    std::fs::create_dir_all(&config_dir)?;
    
    let install_marker = config_dir.join(".installed");
    std::fs::write(&install_marker, "installed")?;
    
    Ok(())
}

/// Check if GUI is available for this installation
pub fn gui_available() -> bool {
    cfg!(feature = "gui")
}

/// Get installation mode preference (for commands that need to know)
#[allow(dead_code)]
pub fn get_install_preference() -> InstallConfig {
    // For existing installations, detect based on available features
    InstallConfig {
        gui_enabled: cfg!(feature = "gui"),
        skip_setup: false,
    }
}