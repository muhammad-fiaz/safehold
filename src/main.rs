mod cli;
mod config;
mod crypto;
mod envops;
mod store;
#[cfg(feature = "gui")]
mod ui;

use anyhow::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("safehold error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cmd = cli::build_cli();
    cli::dispatch(cmd)
}
