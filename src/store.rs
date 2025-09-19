use crate::cli::CreateArgs;
use crate::config::{self, SetMeta};
use crate::crypto;
use anyhow::{bail, Context, Result};
use std::fs;

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

    println!("Created set {} ({})", id, if args.lock {"locked"} else {"unlocked"});
    Ok(())
}

pub fn cmd_list_sets() -> Result<()> {
    let cfg = config::load_config()?;
    println!("ID\tNAME\tLOCKED");
    for s in cfg.sets {
        println!("{}\t{}\t{}", s.id, s.name, s.locked);
    }
    println!("GLOBAL\tglobal\t{}", cfg.global_locked);
    Ok(())
}

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
    println!("Deleted set {id}");
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
            println!("GUI feature not enabled. Rebuild with --features gui");
            return Ok(());
        }
    }
    println!("Use --gui to launch graphical interface.");
    Ok(())
}

pub fn cmd_setup() -> Result<()> {
    // We print PATH guidance; installation scripts differ per OS, keep instructions simple.
    let base = config::ensure_layout()?;
    crypto::ensure_app_key(&base)?;
    println!("Add the cargo bin directory to your PATH to access 'safehold':");
    #[cfg(windows)]
    {
        println!("Windows: setx PATH \"%USERPROFILE%\\.cargo\\bin;%PATH%\"");
    }
    #[cfg(not(windows))]
    {
        println!("Linux/macOS: echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.bashrc && source ~/.bashrc");
    }
    println!("You can create desktop or Start Menu shortcuts to launch the GUI via 'safehold launch --gui' if built with GUI feature.");
    Ok(())
}
