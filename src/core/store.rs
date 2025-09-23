//! Store-level commands: create/list/delete projects, setup, launch
use crate::cli::cli::{CreateArgs, DeleteProjectArgs};
use crate::cli::styles;
use crate::core::config::{self, SetMeta};
use crate::core::crypto;
use anyhow::{Context, Result, bail};
use std::fs;
use std::process::Command;

/// Create a new project (or configure global).
pub fn cmd_create(args: CreateArgs) -> Result<()> {
    let base = config::ensure_layout()?;
    crypto::ensure_app_key(&base)?;

    let mut cfg = config::load_config()?;

    let name = args.name.clone();
    let id = if name == "global" {
        "global".to_string()
    } else {
        config::next_set_id(&name, &cfg.sets)
    };

    let dir = if id == "global" {
        config::global_dir()?
    } else {
        config::set_dir(&id)?
    };
    if dir.exists() {
        bail!("project already exists: {}", id);
    }
    fs::create_dir_all(&dir)?;

    if args.lock {
        let password = match args.password {
            Some(p) => p,
            None => {
                let p = rpassword::prompt_password("Project password: ")?;
                p
            }
        };
        let lock = crypto::create_lock(&password)?;
        fs::write(config::lock_path(&dir), serde_json::to_vec_pretty(&lock)?)?;
    }

    if id != "global" {
        let meta = SetMeta {
            id: id.clone(),
            name: name.clone(),
            locked: args.lock,
        };
        cfg.sets.push(meta);
        config::save_config(&cfg)?;
    } else {
        cfg.global_locked = args.lock;
        config::save_config(&cfg)?;
    }

    styles::success(format!(
        "Created project '{}' ({}) with ID: {}",
        args.name,
        if args.lock {
            "🔒 locked"
        } else {
            "🔓 unlocked"
        },
        id
    ));
    Ok(())
}

/// List all projects (and global).
pub fn cmd_list_sets() -> Result<()> {
    let cfg = config::load_config()?;

    styles::header("SafeHold Projects");
    styles::divider();

    if cfg.sets.is_empty() && !cfg.global_locked {
        styles::info("No projects found. Create one with: safehold create <name>");
        return Ok(());
    }

    // Show global project
    let lock_status = if cfg.global_locked {
        "🔒 Locked"
    } else {
        "🔓 Unlocked"
    };
    styles::kv("GLOBAL", format!("global - {}", lock_status));

    // Show all custom projects
    let project_count = cfg.sets.len();
    for s in cfg.sets {
        let lock_status = if s.locked {
            "🔒 Locked"
        } else {
            "🔓 Unlocked"
        };
        styles::kv(&s.id, format!("{} - {}", s.name, lock_status));
    }

    println!();
    styles::info(format!("Total: {} project(s) + global", project_count));
    Ok(())
}

/// Delete a project by id or the global space.
pub fn cmd_delete_set(args: &DeleteProjectArgs) -> Result<()> {
    let mut cfg = config::load_config()?;
    let (is_global, dir) = if args.id == "global" {
        (true, config::global_dir()?)
    } else {
        (false, config::set_dir(&args.id)?)
    };

    // Check if project exists in config
    let project_exists = if is_global {
        cfg.global_locked
    } else {
        cfg.sets
            .iter()
            .any(|s| s.id == args.id || s.name == args.id)
    };

    if !project_exists && !dir.exists() {
        bail!("❌ Project '{}' not found", args.id);
    }

    // Confirmation prompt (skip if force is true)
    if !args.force {
        use std::io::{self, Write};
        let project_type = if is_global {
            "global storage"
        } else {
            "project"
        };
        styles::warn(format!(
            "⚠️  Delete {} '{}' and all its credentials?",
            project_type, args.id
        ));
        if !is_global {
            styles::warn("⚠️  This will permanently delete all credentials in this project!");
        }
        print!("Confirm (y/N): ");
        io::stdout().flush().unwrap_or(());

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim().to_lowercase();
                if input != "y" && input != "yes" {
                    styles::info("❌ Deletion cancelled");
                    return Ok(());
                }
            }
            Err(_) => {
                styles::info("❌ Deletion cancelled");
                return Ok(());
            }
        }
    }

    // Delete directory if it exists
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("delete {}", dir.display()))?;
    }

    // Update config
    if !is_global {
        cfg.sets.retain(|s| s.id != args.id && s.name != args.id);
    } else {
        cfg.global_locked = false;
    }
    config::save_config(&cfg)?;

    styles::success(format!(
        "🗑️ Deleted {} '{}' and all its credentials",
        if is_global {
            "global storage"
        } else {
            "project"
        },
        args.id
    ));
    Ok(())
}

pub fn cmd_launch(gui: bool) -> Result<()> {
    if gui {
        #[cfg(feature = "gui")]
        {
            // Display ASCII art banner for GUI launch
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
            println!(
                "│                     Version {} - GUI Mode 🖥️                   │",
                env!("CARGO_PKG_VERSION")
            );
            println!("└─────────────────────────────────────────────────────────────────┘");
            println!();

            styles::info("🚀 Launching SafeHold GUI...");
            crate::gui::ui::launch_gui()?;
            return Ok(());
        }
        #[cfg(not(feature = "gui"))]
        {
            styles::error("GUI is not available in this installation");
            styles::info(
                "💡 To get GUI support, reinstall with: cargo install safehold --features gui",
            );
            styles::info("Then run: safehold launch --gui");
            return Ok(());
        }
    }

    styles::header("SafeHold Launch Options");
    if crate::utils::install::gui_available() {
        styles::bullet("🖥️ Launch GUI: safehold launch --gui");
    } else {
        styles::bullet("📝 CLI-only mode: use safehold commands");
        styles::bullet("💡 Add GUI: cargo install safehold --features gui");
    }
    styles::bullet("📚 Help: safehold --help");
    styles::bullet("⚙️ Setup: safehold setup --add-path");
    Ok(())
}

/// Print PATH guidance and ensure base layout is initialized.
pub fn cmd_setup(add_path: bool) -> Result<()> {
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
    println!(
        "│                        Version {} - Setup                       │",
        env!("CARGO_PKG_VERSION")
    );
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    let pb = styles::spinner("🚀 Initializing SafeHold environment...");
    let base = config::ensure_layout()?;
    crypto::ensure_app_key(&base)?;
    styles::finish_spinner_success(pb, "SafeHold environment initialized");

    println!();
    styles::header("PATH Configuration");

    if add_path {
        // Support a dry-run env var for tests to avoid mutating user PATH.
        if std::env::var("SAFEHOLD_PATH_DRY_RUN").ok().as_deref() == Some("1") {
            styles::success("PATH update (dry run) would be applied.");
            styles::info("To actually update PATH, run without SAFEHOLD_PATH_DRY_RUN.");
        } else {
            #[cfg(windows)]
            {
                // Update PATH persistently using setx. This appends cargo bin to PATH.
                let cargo_bin = format!(
                    "{}\\.cargo\\bin",
                    std::env::var("USERPROFILE").unwrap_or_else(|_| "%USERPROFILE%".into())
                );
                let cmdline = format!("setx PATH \"{};%PATH%\"", cargo_bin);
                let status = Command::new("cmd").arg("/C").arg(cmdline).status();
                match status {
                    Ok(st) if st.success() => {
                        styles::success("PATH updated for current user");
                        styles::info(
                            "⚠️ You may need to restart your terminal for changes to take effect",
                        );
                    }
                    Ok(st) => styles::warn(format!(
                        "PATH update returned status {}. You may need admin privileges.",
                        st
                    )),
                    Err(e) => styles::error(format!("Failed to update PATH automatically: {}", e)),
                }
            }
            #[cfg(not(windows))]
            {
                // Append export to ~/.bashrc
                let home = std::env::var("HOME").unwrap_or_else(|_| "$HOME".into());
                let bashrc = format!("{}/.bashrc", home);
                let line = "export PATH=\"$HOME/.cargo/bin:$PATH\"\n";
                match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&bashrc)
                {
                    Ok(mut f) => {
                        use std::io::Write;
                        if let Err(e) = f.write_all(line.as_bytes()) {
                            styles::error(format!("Failed to append to {}: {}", bashrc, e));
                        } else {
                            styles::success(format!("Added cargo bin to PATH in {}.", bashrc));
                            styles::info("🔄 Restart your shell or run: source ~/.bashrc");
                        }
                    }
                    Err(e) => styles::error(format!("Failed to open {}: {}", bashrc, e)),
                }
            }
        }
    } else {
        styles::info("💡 Manual PATH setup:");
        #[cfg(windows)]
        {
            styles::bullet("Windows: setx PATH \"%USERPROFILE%\\.cargo\\bin;%PATH%\"");
        }
        #[cfg(not(windows))]
        {
            styles::bullet(
                "Linux/macOS: echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.bashrc && source ~/.bashrc",
            );
        }
        styles::info("Or run: safehold setup --add-path");
    }

    println!();
    if crate::utils::install::gui_available() {
        styles::info("🖥️ GUI available! Create shortcuts to launch with: safehold launch --gui");
    } else {
        styles::info(
            "📝 CLI-only installation. Reinstall with GUI: cargo install safehold --features gui",
        );
    }
    Ok(())
}
