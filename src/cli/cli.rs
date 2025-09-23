//! CLI schema and dispatch for SafeHold.
use anyhow::Result;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum, ValueHint};

/// Top-level CLI options and subcommands.
#[derive(Parser, Debug)]
#[command(
    name = "safehold",
    version,
    about = "🔐 SafeHold - Secure Cross-Platform Credential Manager",
    long_about = "🔐 SafeHold - Professional-grade credential manager with military-grade encryption, secure storage, and both CLI and GUI interfaces.",
    arg_required_else_help = true,
    help_template = "{before-help}{name} {version}\n{about}\n\n{usage-heading} {usage}\n\n{all-args}{after-help}"
)]
pub struct Cli {
    /// Force color choices for output formatting.
    #[arg(global=true, long, value_enum, default_value_t=ColorChoice::Auto, help = "🎨 Control colored output")]
    pub color: ColorChoice,
    /// Style of output: fancy (spinners) or plain.
    #[arg(global=true, long, value_enum, default_value_t=StyleChoice::Fancy, help = "✨ Output style: fancy (spinners) or plain")]
    pub style: StyleChoice,
    /// Quiet mode: suppress non-essential output.
    #[arg(global=true, long, action=ArgAction::SetTrue, help = "🤫 Suppress non-essential output")]
    pub quiet: bool,
    /// Install with GUI support (for installation)
    #[arg(long, global = true, hide = true)]
    pub gui: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
pub enum StyleChoice {
    Fancy,
    Plain,
}

/// All subcommands supported by SafeHold.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 📁 Create a new credential project (unlocked by default)
    #[command(visible_alias = "new")]
    Create(CreateArgs),
    /// 📋 List all credential projects
    #[command(visible_aliases = &["ls", "projects"])]
    ListProjects,
    /// 🗑️ Delete a credential project by ID or name
    #[command(visible_aliases = &["rm", "remove"])]
    DeleteProject(DeleteProjectArgs),
    /// ➕ Add a key/value credential into a project
    #[command(visible_alias = "set")]
    Add(ProjectKeyValueArgs),
    /// 🔍 Get a credential value from a project
    #[command(visible_alias = "show")]
    Get(ProjectKeyArgs),
    /// 📝 List all credentials in a project
    #[command(visible_alias = "keys")]
    List(ProjectTargetArgs),
    /// ❌ Delete a credential from a project
    #[command(visible_aliases = &["del", "rm-key"])]
    Delete(ProjectKeyArgsForce),
    /// ✏️ Update/modify a credential value in a project
    #[command(visible_aliases = &["modify", "change", "edit"])]
    Update(ProjectKeyValueArgs),
    /// 📊 Count credentials in projects
    #[command(visible_alias = "total")]
    Count(CountArgs),
    /// ➕ Add a key/value credential to global storage
    #[command(name = "global-add", visible_aliases = &["gadd", "global-set"])]
    GlobalAdd(GlobalKeyValueArgs),
    /// 🔍 Get a credential value from global storage
    #[command(name = "global-get", visible_aliases = &["gget", "global-show"])]
    GlobalGet(GlobalKeyArgs),
    /// 📝 List all credentials in global storage
    #[command(name = "global-list", visible_aliases = &["glist", "global-keys"])]
    GlobalList,
    /// ❌ Delete a credential from global storage
    #[command(name = "global-delete", visible_aliases = &["gdel", "global-rm"])]
    GlobalDelete(GlobalKeyArgsForce),
    /// ✏️ Update/modify a credential value in global storage
    #[command(name = "global-update", visible_aliases = &["gupdate", "global-modify"])]
    GlobalUpdate(GlobalKeyValueArgs),
    /// 📤 Export credentials to .env format
    Export(ExportArgs),
    /// 🚀 Run a command with credentials as environment variables (no file written)
    #[command(visible_alias = "exec")]
    Run(RunArgs),
    /// 🔍 Show all projects and their credentials (will prompt for locked)
    #[command(name = "show-all", visible_alias = "all")]
    ShowAll,
    /// 🧹 Clean up stray plaintext .env files in current directory tree
    Clean,
    /// �️ Clean cache and temporary files
    #[command(name = "clean-cache", visible_aliases = &["clear-cache", "cache-clean"])]
    CleanCache {
        #[arg(long, action=ArgAction::SetTrue, help = "🚨 Skip confirmation prompt")]
        force: bool,
    },
    /// 💥 Delete ALL projects and data (DESTRUCTIVE!)
    #[command(name = "delete-all", visible_aliases = &["clear-all", "nuke"])]
    DeleteAll {
        #[arg(long, action=ArgAction::SetTrue, help = "🚨 Skip confirmation prompt (DANGEROUS!)")]
        force: bool,
    },
    /// ℹ️ Show application information and details
    #[command(visible_alias = "info")]
    About,
    /// 🔐 Manage Global Master Lock - unified password for ALL projects
    #[command(name = "master-lock", visible_aliases = &["mlock", "global-master"])]
    MasterLock {
        #[arg(long, action=ArgAction::SetTrue, help = "🔒 Enable Global Master Lock")]
        enable: bool,
        #[arg(long, action=ArgAction::SetTrue, help = "🔓 Disable Global Master Lock")]
        disable: bool,
    },
    /// �🖥️ Launch SafeHold GUI (if available)
    #[command(visible_alias = "gui")]
    Launch {
        #[arg(long, action=ArgAction::SetTrue, help = "🚀 Force launch GUI mode")]
        gui: bool,
    },
    /// ⚙️ Setup SafeHold environment and PATH configuration
    Setup {
        #[arg(long, action=ArgAction::SetTrue, help = "🛤️ Automatically add SafeHold to system PATH")]
        add_path: bool,
    },
    /// 🔄 Check for SafeHold updates from crates.io
    #[command(name = "check-update", visible_aliases = &["update-check", "check-updates"])]
    CheckUpdate,
}

/// Args for `create` command.
#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Project name, or "global" for the global project
    #[arg(help = "📛 Project name (use 'global' for the default global project)")]
    pub name: String,
    /// Create as locked with password prompt
    #[arg(long, short='l', action=ArgAction::SetTrue, help = "🔒 Create project with password protection")]
    pub lock: bool,
    /// Provide password non-interactively (unsafe on shared shells)
    #[arg(long, value_hint=ValueHint::Other, help = "🔑 Set password non-interactively (⚠️ unsafe on shared shells)")]
    pub password: Option<String>,
}

/// Target-only arg wrapper for commands that operate on a project.
#[derive(Args, Debug)]
pub struct ProjectTargetArgs {
    /// Project ID or name
    #[arg(long, short = 'p', help = "📁 Project ID or name")]
    pub project: String,
}

/// Args for deleting a project.
#[derive(Args, Debug)]
pub struct DeleteProjectArgs {
    /// Project ID or name
    #[arg(help = "📁 Project ID or name")]
    pub id: String,
    /// Skip confirmation prompt
    #[arg(long, action=ArgAction::SetTrue, help = "🚨 Skip confirmation prompt")]
    pub force: bool,
}

/// Args for commands that need a project and a key.
#[derive(Args, Debug)]
pub struct ProjectKeyArgs {
    /// Project ID or name
    #[arg(long, short = 'p', help = "📁 Project ID or name")]
    pub project: String,
    /// Key name
    #[arg(long, short = 'k', help = "🔑 Credential key name")]
    pub key: String,
}

/// Args for commands that need a project and a key with force option.
#[derive(Args, Debug)]
pub struct ProjectKeyArgsForce {
    /// Project ID or name
    #[arg(long, short = 'p', help = "📁 Project ID or name")]
    pub project: String,
    /// Key name
    #[arg(long, short = 'k', help = "🔑 Credential key name")]
    pub key: String,
    /// Skip confirmation prompt
    #[arg(long, action=ArgAction::SetTrue, help = "🚨 Skip confirmation prompt")]
    pub force: bool,
}

/// Args for commands that set a key to a value.
#[derive(Args, Debug)]
pub struct ProjectKeyValueArgs {
    /// Project ID or name
    #[arg(long, short = 'p', help = "📁 Project ID or name")]
    pub project: String,
    /// Key name
    #[arg(long, short = 'k', help = "🔑 Credential key name")]
    pub key: String,
    /// Value (omit to read from stdin)
    #[arg(
        long,
        short = 'v',
        help = "💎 Credential value (omit to read from stdin securely)"
    )]
    pub value: Option<String>,
}

/// Args for exporting .env files.
#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Project ID or name; omit with --global for global
    #[arg(
        long,
        short = 'p',
        help = "📁 Project ID or name (omit with --global for global project)"
    )]
    pub project: Option<String>,
    /// Export global creds
    #[arg(long, action=ArgAction::SetTrue, help = "🌍 Export from global project")]
    pub global: bool,
    /// Custom filename
    #[arg(long, help = "📄 Custom output filename")]
    pub file: Option<String>,
    /// Overwrite existing file
    #[arg(long, action=ArgAction::SetTrue, help = "🔄 Overwrite existing file if present")]
    pub force: bool,
    /// Create temp file and delete on exit
    #[arg(long, action=ArgAction::SetTrue, help = "⏱️ Create temporary file that gets deleted on exit")]
    pub temp: bool,
}

/// Args for running a process with injected environment variables.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Project ID or name
    #[arg(long, short = 'p', help = "📁 Project ID or name")]
    pub project: String,
    /// Merge in global
    #[arg(long, action=ArgAction::SetTrue, help = "🌍 Merge in credentials from global project")]
    pub with_global: bool,
    /// Command to run after '--'
    #[arg(
        last = true,
        required = true,
        help = "🚀 Command to execute (place after '--')"
    )]
    pub command: Vec<String>,
}

/// Args for count command.
///
/// Provides flexible credential counting with options for:
/// - Counting specific projects or all projects
/// - Including/excluding global credentials
/// - Detailed breakdown per project
#[derive(Args, Debug)]
pub struct CountArgs {
    /// Project ID or name (omit to count all projects)
    #[arg(
        long,
        short = 'p',
        help = "📁 Project ID or name (omit to count all projects)"
    )]
    pub project: Option<String>,
    /// Include global in total count
    #[arg(long, action=ArgAction::SetTrue, help = "🌍 Include global credentials in count")]
    pub include_global: bool,
    /// Show detailed breakdown per project
    #[arg(long, action=ArgAction::SetTrue, help = "📊 Show detailed count breakdown")]
    pub detailed: bool,
}

/// Args for global key operations.
///
/// Used for operations that only need a key name for global credential storage,
/// such as getting or deleting global credentials.
#[derive(Args, Debug)]
pub struct GlobalKeyArgs {
    /// Key name
    #[arg(long, short = 'k', help = "🔑 Credential key name")]
    pub key: String,
}

/// Args for global key operations with force option.
///
/// Used for operations that only need a key name for global credential storage,
/// such as getting or deleting global credentials.
#[derive(Args, Debug)]
pub struct GlobalKeyArgsForce {
    /// Key name
    #[arg(long, short = 'k', help = "🔑 Credential key name")]
    pub key: String,
    /// Skip confirmation prompt
    #[arg(long, action=ArgAction::SetTrue, help = "🚨 Skip confirmation prompt")]
    pub force: bool,
}

/// Args for global key-value operations.
///
/// Used for operations that need both a key and value for global credential storage,
/// such as adding or updating global credentials. The value can be provided via
/// command line or prompted from stdin if omitted.
#[derive(Args, Debug)]
pub struct GlobalKeyValueArgs {
    /// Key name
    #[arg(long, short = 'k', help = "🔑 Credential key name")]
    pub key: String,
    /// Value (omit to read from stdin)
    #[arg(
        long,
        short = 'v',
        help = "💎 Credential value (omit to read from stdin securely)"
    )]
    pub value: Option<String>,
}

/// Parse CLI from process arguments.
pub fn build_cli() -> Cli {
    Cli::parse()
}

/// Dispatch a parsed CLI to the appropriate handler.
pub async fn dispatch(cli: Cli) -> Result<()> {
    // Initialize styles from global flags
    let use_color = match cli.color {
        ColorChoice::Auto => atty::is(atty::Stream::Stdout),
        ColorChoice::Always => true,
        ColorChoice::Never => false,
    };
    let mode = match cli.style {
        StyleChoice::Fancy => crate::cli::styles::RenderMode::Fancy,
        StyleChoice::Plain => crate::cli::styles::RenderMode::Plain,
    };
    crate::cli::styles::init(crate::cli::styles::StyleOptions {
        mode,
        use_color,
        quiet: cli.quiet,
    });
    match cli.command {
        Commands::Create(args) => crate::core::store::cmd_create(args),
        Commands::ListProjects => crate::core::store::cmd_list_sets(),
        Commands::DeleteProject(args) => crate::core::store::cmd_delete_set(&args),
        Commands::Add(args) => crate::operations::envops::cmd_add(args),
        Commands::Get(args) => crate::operations::envops::cmd_get(args),
        Commands::List(args) => crate::operations::envops::cmd_list(args),
        Commands::Delete(args) => crate::operations::envops::cmd_delete(args),
        Commands::Update(args) => crate::operations::envops::cmd_update(args),
        Commands::Count(args) => crate::operations::envops::cmd_count(args),
        Commands::GlobalAdd(args) => crate::operations::envops::cmd_global_add(args),
        Commands::GlobalGet(args) => crate::operations::envops::cmd_global_get(args),
        Commands::GlobalList => crate::operations::envops::cmd_global_list(),
        Commands::GlobalDelete(args) => crate::operations::envops::cmd_global_delete(args),
        Commands::GlobalUpdate(args) => crate::operations::envops::cmd_global_update(args),
        Commands::Export(args) => crate::operations::envops::cmd_export(args),
        Commands::Run(args) => crate::operations::envops::cmd_run(args),
        Commands::ShowAll => crate::operations::envops::cmd_show_all(),
        Commands::Clean => crate::operations::envops::cmd_clean(),
        Commands::CleanCache { force } => crate::operations::envops::cmd_clean_cache(force),
        Commands::DeleteAll { force } => crate::operations::envops::cmd_delete_all(force),
        Commands::About => crate::operations::envops::cmd_about(),
        Commands::MasterLock { enable, disable } => {
            let action = if enable {
                Some(true)
            } else if disable {
                Some(false)
            } else {
                None
            };
            crate::operations::master_lock::cmd_master_lock(action)
        }
        Commands::Launch { gui } => crate::core::store::cmd_launch(gui),
        Commands::Setup { add_path } => crate::core::store::cmd_setup(add_path),
        Commands::CheckUpdate => {
            crate::utils::update_checker::display_cli_update_check().await;
            Ok(())
        }
    }
}
