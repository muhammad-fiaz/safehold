mod cli;
mod config;
mod crypto;
mod envops;
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
    cli::dispatch(cmd)
}
