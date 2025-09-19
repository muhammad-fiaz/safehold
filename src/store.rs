//! Store-level commands: create/list/delete sets, setup, launch
use crate::cli::CreateArgs;
use crate::config::{self, SetMeta};
use crate::crypto;
use anyhow::{bail, Context, Result};
use std::fs;
use crate::styles;
use std::process::Command;

/// Create a new set (or configure global).
pub fn cmd_create(args: CreateArgs) -> Result<()> {
    let base = config::ensure_layout()?;
    crypto::ensure_app_key(&base)?;

    let mut cfg = config::load_config()?;

    let name = args.name.clone();
    let id = if name == "global" { "global".to_string() } else { config::next_set_id(&name, &cfg.sets) };

    let dir = if id == "global" { config::global_dir()? } else { config::set_dir(&id)? };
    if dir.exists() { bail!("set already exists: {}", id); }
    fs::create_dir_all(&dir)?;

    if args.lock {
        let password = match args.password {
            Some(p) => p,
            None => {
                let p = rpassword::prompt_password("Set password: ")?;
                p
            }
        };
        let lock = crypto::create_lock(&password)?;
        fs::write(config::lock_path(&dir), serde_json::to_vec_pretty(&lock)?)?;
    }

    if id != "global" {
        let meta = SetMeta { id: id.clone(), name: name.clone(), locked: args.lock };
        cfg.sets.push(meta);
        config::save_config(&cfg)?;
    } else {
        cfg.global_locked = args.lock;
        config::save_config(&cfg)?;
    }

    styles::ok(format!("Created set {} ({})", id, if args.lock {"locked"} else {"unlocked"}));
    Ok(())
}

/// List all sets (and global).
pub fn cmd_list_sets() -> Result<()> {
    let cfg = config::load_config()?;
    styles::info("ID\tNAME\tLOCKED");
    for s in cfg.sets {
        println!("{}\t{}\t{}", s.id, s.name, s.locked);
    }
    println!("GLOBAL\tglobal\t{}", cfg.global_locked);
    Ok(())
}

/// Delete a set by id or the global space.
pub fn cmd_delete_set(id: &str) -> Result<()> {
    let mut cfg = config::load_config()?;
    let (is_global, dir) = if id == "global" { (true, config::global_dir()?) } else { (false, config::set_dir(id)?) };
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("delete {}", dir.display()))?;
    }
    if !is_global {
        cfg.sets.retain(|s| s.id != id && s.name != id);
    } else {
        cfg.global_locked = false;
    }
    config::save_config(&cfg)?;
    styles::ok(format!("Deleted set {id}"));
    Ok(())
}

pub fn cmd_launch(gui: bool) -> Result<()> {
    if gui {
        #[cfg(feature="gui")]
        {
            crate::ui::launch_gui()?;
            return Ok(());
        }
        #[cfg(not(feature="gui"))]
        {
            styles::warn("GUI is not installed. Reinstall with: cargo install safehold --features gui");
            styles::info("Then run: safehold launch --gui");
            return Ok(());
        }
    }
    styles::info("Use --gui to launch graphical interface.");
    Ok(())
}

/// Print PATH guidance and ensure base layout is initialized.
pub fn cmd_setup(add_path: bool) -> Result<()> {
    // We print PATH guidance; installation scripts differ per OS, keep instructions simple.
    let pb = styles::spinner("Initializing...");
    let base = config::ensure_layout()?;
    crypto::ensure_app_key(&base)?;
    styles::finish_spinner(pb, "Initialized");
    styles::info("Add the cargo bin directory to your PATH to access 'safehold':");
    #[cfg(windows)]
    {
        println!("Windows: setx PATH \"%USERPROFILE%\\.cargo\\bin;%PATH%\"");
    }
    #[cfg(not(windows))]
    {
        println!("Linux/macOS: echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.bashrc && source ~/.bashrc");
    }
    if add_path {
        // Support a dry-run env var for tests to avoid mutating user PATH.
        if std::env::var("SAFEHOLD_PATH_DRY_RUN").ok().as_deref() == Some("1") {
            styles::ok("PATH update (dry run) would be applied.");
            styles::info("To actually update PATH, run without SAFEHOLD_PATH_DRY_RUN.");
        } else {
            #[cfg(windows)]
            {
                // Update PATH persistently using setx. This appends cargo bin to PATH.
                let cargo_bin = format!("{}\\.cargo\\bin", std::env::var("USERPROFILE").unwrap_or_else(|_| "%USERPROFILE%".into()));
                let cmdline = format!("setx PATH \"{};%PATH%\"", cargo_bin);
                let status = Command::new("cmd").arg("/C").arg(cmdline).status();
                match status {
                    Ok(st) if st.success() => styles::ok("PATH updated for current user (You may need to restart terminals)."),
                    Ok(st) => styles::warn(format!("PATH update returned status {}. You may need admin privileges.", st)),
                    Err(e) => styles::warn(format!("Failed to update PATH automatically: {}", e)),
                }
            }
            #[cfg(not(windows))]
            {
                // Append export to ~/.bashrc
                let home = std::env::var("HOME").unwrap_or_else(|_| "$HOME".into());
                let bashrc = format!("{}/.bashrc", home);
                let line = "export PATH=\"$HOME/.cargo/bin:$PATH\"\n";
                match std::fs::OpenOptions::new().create(true).append(true).open(&bashrc) {
                    Ok(mut f) => {
                        use std::io::Write;
                        if let Err(e) = f.write_all(line.as_bytes()) {
                            styles::warn(format!("Failed to append to {}: {}", bashrc, e));
                        } else {
                            styles::ok(format!("Added cargo bin to PATH in {}.", bashrc));
                            styles::info("Restart your shell or 'source ~/.bashrc'.");
                        }
                    }
                    Err(e) => styles::warn(format!("Failed to open {}: {}", bashrc, e)),
                }
            }
        }
    }
    styles::info("You can create desktop or Start Menu shortcuts to launch the GUI via 'safehold launch --gui' if installed with GUI feature.");
    Ok(())
}
