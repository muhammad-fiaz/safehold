//! CLI schema and dispatch for SafeHold.
use clap::{ArgAction, Parser, Subcommand, Args, ValueHint, ValueEnum};
use anyhow::Result;

/// Top-level CLI options and subcommands.
#[derive(Parser, Debug)]
#[command(name = "safehold", version, about = "Secure credentials manager (CLI + GUI)", long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Force color choices for output formatting.
    #[arg(global=true, long, value_enum, default_value_t=ColorChoice::Auto)]
    pub color: ColorChoice,
    /// Style of output: fancy (spinners) or plain.
    #[arg(global=true, long, value_enum, default_value_t=StyleChoice::Fancy)]
    pub style: StyleChoice,
    /// Quiet mode: suppress non-essential output.
    #[arg(global=true, long, action=ArgAction::SetTrue)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum ColorChoice { Auto, Always, Never }

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum StyleChoice { Fancy, Plain }

/// All subcommands supported by SafeHold.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new credential project (unlocked by default)
    Create(CreateArgs),
    /// List credential projects
    ListProjects,
    /// Delete a credential project by ID or name
    DeleteProject { id: String },
    /// Add a key/value into a project
    Add(ProjectKeyValueArgs),
    /// Get a value from a project
    Get(ProjectKeyArgs),
    /// List all keys in a project
    List(ProjectTargetArgs),
    /// Delete a key in a project
    Delete(ProjectKeyArgs),
    /// Export credentials to .env
    Export(ExportArgs),
    /// Run a command with env vars injected (no file written)
    Run(RunArgs),
    /// Show all projects and their keys (will prompt for locked)
    ShowAll,
    /// Clean stray plaintext .env files in current tree
    Clean,
    /// Launch GUI
    Launch { #[arg(long, action=ArgAction::SetTrue)] gui: bool },
    /// Setup: print PATH guidance and optionally add cargo bin to PATH
    Setup { #[arg(long, action=ArgAction::SetTrue)] add_path: bool },
}

/// Args for `create` command.
#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Project name, or "global" for the global project
    pub name: String,
    /// Create as locked with password prompt
    #[arg(long, short='l', action=ArgAction::SetTrue)]
    pub lock: bool,
    /// Provide password non-interactively (unsafe on shared shells)
    #[arg(long, value_hint=ValueHint::Other)]
    pub password: Option<String>,
}

/// Target-only arg wrapper for commands that operate on a project.
#[derive(Args, Debug)]
pub struct ProjectTargetArgs {
    /// Project ID or name
    #[arg(long, short='p')]
    pub project: String,
}

/// Args for commands that need a project and a key.
#[derive(Args, Debug)]
pub struct ProjectKeyArgs {
    /// Project ID or name
    #[arg(long, short='p')]
    pub project: String,
    /// Key name
    #[arg(long, short='k')]
    pub key: String,
}

/// Args for commands that set a key to a value.
#[derive(Args, Debug)]
pub struct ProjectKeyValueArgs {
    /// Project ID or name
    #[arg(long, short='p')]
    pub project: String,
    /// Key name
    #[arg(long, short='k')]
    pub key: String,
    /// Value (omit to read from stdin)
    #[arg(long, short='v')]
    pub value: Option<String>,
}

/// Args for exporting .env files.
#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Project ID or name; omit with --global for global
    #[arg(long, short='p')]
    pub project: Option<String>,
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

/// Args for running a process with injected environment variables.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Project ID or name
    #[arg(long, short='p')]
    pub project: String,
    /// Merge in global
    #[arg(long, action=ArgAction::SetTrue)]
    pub with_global: bool,
    /// Command to run after '--'
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

/// Parse CLI from process arguments.
pub fn build_cli() -> Cli { Cli::parse() }

/// Dispatch a parsed CLI to the appropriate handler.
pub fn dispatch(cli: Cli) -> Result<()> {
    // Initialize styles from global flags
    let use_color = match cli.color { ColorChoice::Auto => atty::is(atty::Stream::Stdout), ColorChoice::Always => true, ColorChoice::Never => false };
    let mode = match cli.style { StyleChoice::Fancy => crate::styles::RenderMode::Fancy, StyleChoice::Plain => crate::styles::RenderMode::Plain };
    crate::styles::init(crate::styles::StyleOptions { mode, use_color, quiet: cli.quiet });
    match cli.command {
        Commands::Create(args) => crate::store::cmd_create(args),
        Commands::ListProjects => crate::store::cmd_list_sets(),
        Commands::DeleteProject { id } => crate::store::cmd_delete_set(&id),
        Commands::Add(args) => crate::envops::cmd_add(args),
        Commands::Get(args) => crate::envops::cmd_get(args),
        Commands::List(args) => crate::envops::cmd_list(args),
        Commands::Delete(args) => crate::envops::cmd_delete(args),
        Commands::Export(args) => crate::envops::cmd_export(args),
        Commands::Run(args) => crate::envops::cmd_run(args),
        Commands::ShowAll => crate::envops::cmd_show_all(),
        Commands::Clean => crate::envops::cmd_clean(),
        Commands::Launch { gui } => crate::store::cmd_launch(gui),
        Commands::Setup { add_path } => crate::store::cmd_setup(add_path),
    }
}
