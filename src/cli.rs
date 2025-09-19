use clap::{ArgAction, Parser, Subcommand, Args, ValueHint};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "safehold", version, about = "Secure credentials manager (CLI + GUI)", long_about = None, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new credential set (unlocked by default)
    Create(CreateArgs),
    /// List credential sets
    ListSets,
    /// Delete a credential set by ID or name
    DeleteSet { id: String },
    /// Add a key/value into a set
    Add(SetKeyValueArgs),
    /// Get a value from a set
    Get(SetKeyArgs),
    /// List all keys in a set
    List(SetTargetArgs),
    /// Delete a key in a set
    Delete(SetKeyArgs),
    /// Export credentials to .env
    Export(ExportArgs),
    /// Run a command with env vars injected (no file written)
    Run(RunArgs),
    /// Show all sets and their keys (will prompt for locked)
    ShowAll,
    /// Clean stray plaintext .env files in current tree
    Clean,
    /// Launch GUI
    Launch { #[arg(long, action=ArgAction::SetTrue)] gui: bool },
    /// Setup: print PATH instructions and optionally install launcher
    Setup,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Set name, or "global" for the global set
    pub name: String,
    /// Create as locked with password prompt
    #[arg(long, short='l', action=ArgAction::SetTrue)]
    pub lock: bool,
    /// Provide password non-interactively (unsafe on shared shells)
    #[arg(long, value_hint=ValueHint::Other)]
    pub password: Option<String>,
}

#[derive(Args, Debug)]
pub struct SetTargetArgs {
    /// Set ID or name
    #[arg(long, short='s')]
    pub set: String,
}

#[derive(Args, Debug)]
pub struct SetKeyArgs {
    /// Set ID or name
    #[arg(long, short='s')]
    pub set: String,
    /// Key name
    #[arg(long, short='k')]
    pub key: String,
}

#[derive(Args, Debug)]
pub struct SetKeyValueArgs {
    /// Set ID or name
    #[arg(long, short='s')]
    pub set: String,
    /// Key name
    #[arg(long, short='k')]
    pub key: String,
    /// Value (omit to read from stdin)
    #[arg(long, short='v')]
    pub value: Option<String>,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Set ID or name; omit with --global for global
    #[arg(long, short='s')]
    pub set: Option<String>,
    /// Export global creds
    #[arg(long, action=ArgAction::SetTrue)]
    pub global: bool,
    /// Custom filename
    #[arg(long)]
    pub file: Option<String>,
    /// Overwrite existing file
    #[arg(long, action=ArgAction::SetTrue)]
    pub force: bool,
    /// Create temp file and delete on exit
    #[arg(long, action=ArgAction::SetTrue)]
    pub temp: bool,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Set ID or name
    #[arg(long, short='s')]
    pub set: String,
    /// Merge in global
    #[arg(long, action=ArgAction::SetTrue)]
    pub with_global: bool,
    /// Command to run after '--'
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

pub fn build_cli() -> Cli { Cli::parse() }

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Create(args) => crate::store::cmd_create(args),
        Commands::ListSets => crate::store::cmd_list_sets(),
        Commands::DeleteSet { id } => crate::store::cmd_delete_set(&id),
        Commands::Add(args) => crate::envops::cmd_add(args),
        Commands::Get(args) => crate::envops::cmd_get(args),
        Commands::List(args) => crate::envops::cmd_list(args),
        Commands::Delete(args) => crate::envops::cmd_delete(args),
        Commands::Export(args) => crate::envops::cmd_export(args),
        Commands::Run(args) => crate::envops::cmd_run(args),
        Commands::ShowAll => crate::envops::cmd_show_all(),
        Commands::Clean => crate::envops::cmd_clean(),
        Commands::Launch { gui } => crate::store::cmd_launch(gui),
        Commands::Setup => crate::store::cmd_setup(),
    }
}
