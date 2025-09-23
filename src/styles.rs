//! Styling helpers for colorful output and rich formatting.
//!
//! Provides a Python rich library-like experience with colors, emojis, boxes,
//! progress bars, and sophisticated formatting. Use `init(opts)` once at startup,
//! then use the helpers for consistent, beautiful output.

use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::OnceCell;
use owo_colors::OwoColorize;

/// Controls how CLI renders output
#[derive(Clone, Copy, Debug)]
pub enum RenderMode {
    /// Colors, emojis, and spinners enabled
    Fancy,
    /// Minimal: no color, no emoji, no spinner
    Plain,
}

#[derive(Clone, Copy, Debug)]
pub struct StyleOptions {
    pub mode: RenderMode,
    pub use_color: bool,
    pub quiet: bool,
}

static STYLE_OPTS: OnceCell<StyleOptions> = OnceCell::new();

/// Initialize style options once.
pub fn init(opts: StyleOptions) {
    let _ = STYLE_OPTS.set(opts);
}

fn opts() -> StyleOptions {
    *STYLE_OPTS.get().unwrap_or(&StyleOptions {
        mode: RenderMode::Fancy,
        use_color: true,
        quiet: false,
    })
}

/// Success messages with green checkmark
pub fn success<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        println!("{} {}", "✅".green().bold(), m.green().bold());
    } else if opts().use_color {
        println!("{} {}", "✔".green().bold(), m.green());
    } else {
        println!("[OK] {}", m);
    }
}

/// General OK messages (kept for compatibility)
pub fn ok<S: AsRef<str>>(msg: S) {
    success(msg);
}

/// Info messages with blue information icon
pub fn info<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        println!("{} {}", "ℹ️".blue().bold(), m.cyan());
    } else if opts().use_color {
        println!("{} {}", "ℹ".blue().bold(), m);
    } else {
        println!("[INFO] {}", m);
    }
}

/// Warning messages with yellow warning icon
pub fn warn<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        eprintln!("{} {}", "⚠️".yellow().bold(), m.yellow().bold());
    } else if opts().use_color {
        eprintln!("{} {}", "⚠".yellow().bold(), m.yellow());
    } else {
        eprintln!("[WARN] {}", m);
    }
}

/// Error messages with red cross icon
pub fn error<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        eprintln!("{} {}", "❌".red().bold(), m.red().bold());
    } else if opts().use_color {
        eprintln!("{} {}", "✘".red().bold(), m.red());
    } else {
        eprintln!("[ERR] {}", m);
    }
}

/// Alias for compatibility
#[allow(dead_code)]
pub fn err<S: AsRef<str>>(msg: S) {
    error(msg);
}

/// Debug messages with purple debug icon
#[allow(dead_code)]
pub fn debug<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        println!("{} {}", "🐛".magenta().bold(), m.magenta());
    } else if opts().use_color {
        println!("{} {}", "D".magenta().bold(), m.magenta());
    } else {
        println!("[DEBUG] {}", m);
    }
}

/// Print a header with decorative border
pub fn header<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    let line = "─".repeat(m.len() + 4);
    
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        println!("{}", format!("┌{}┐", line).cyan().bold());
        println!("{}", format!("│ {} │", m).cyan().bold());
        println!("{}", format!("└{}┘", line).cyan().bold());
    } else if opts().use_color {
        println!("{}", format!("╭{}╮", line).cyan());
        println!("{}", format!("│ {} │", m).cyan());
        println!("{}", format!("╰{}╯", line).cyan());
    } else {
        println!("{}", format!("+{}+", line.replace("─", "-")));
        println!("| {} |", m);
        println!("{}", format!("+{}+", line.replace("─", "-")));
    }
}

/// Print a simple divider
pub fn divider() {
    if opts().quiet {
        return;
    }
    if opts().use_color {
        println!("{}", "─".repeat(50).bright_black());
    } else {
        println!("{}", "-".repeat(50));
    }
}

/// Print key-value pairs with nice formatting
pub fn kv<K: AsRef<str>, V: AsRef<str>>(key: K, value: V) {
    if opts().quiet {
        return;
    }
    let k = key.as_ref();
    let v = value.as_ref();
    
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        println!("  {} {}", format!("{}:", k).blue().bold(), v.white());
    } else if opts().use_color {
        println!("  {}: {}", k.blue(), v);
    } else {
        println!("  {}: {}", k, v);
    }
}

/// Print a bullet point item
pub fn bullet<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    
    if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
        println!("  {} {}", "•".cyan().bold(), m);
    } else if opts().use_color {
        println!("  {} {}", "•".cyan(), m);
    } else {
        println!("  * {}", m);
    }
}

/// Create a beautiful spinner with rich styling
pub fn spinner<S: AsRef<str>>(msg: S) -> ProgressBar {
    let o = opts();
    if matches!(o.mode, RenderMode::Fancy) && !o.quiet {
        let pb = ProgressBar::new_spinner();
        let sty = if o.use_color {
            ProgressStyle::with_template("{spinner:.cyan} {wide_msg:.cyan}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        } else {
            ProgressStyle::default_spinner()
        };
        pb.set_style(sty);
        pb.enable_steady_tick(std::time::Duration::from_millis(120));
        pb.set_message(msg.as_ref().to_string());
        pb
    } else {
        ProgressBar::hidden()
    }
}

/// Create a progress bar for operations with known total
#[allow(dead_code)]
pub fn progress_bar(total: u64) -> ProgressBar {
    let o = opts();
    if matches!(o.mode, RenderMode::Fancy) && !o.quiet {
        let pb = ProgressBar::new(total);
        let sty = if o.use_color {
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg:.cyan}"
            ).unwrap()
        } else {
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40} {pos:>7}/{len:7} {msg}"
            ).unwrap()
        };
        pb.set_style(sty);
        pb
    } else {
        ProgressBar::hidden()
    }
}

/// Finish spinner with success message
pub fn finish_spinner_success(pb: ProgressBar, msg: &str) {
    if pb.is_hidden() {
        success(msg);
    } else {
        let final_msg = if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
            format!("{} {}", "✅".green(), msg.green().bold())
        } else {
            format!("✔ {}", msg)
        };
        pb.finish_with_message(final_msg);
    }
}

/// Finish spinner with error message
#[allow(dead_code)]
pub fn finish_spinner_error(pb: ProgressBar, msg: &str) {
    if pb.is_hidden() {
        error(msg);
    } else {
        let final_msg = if opts().use_color && matches!(opts().mode, RenderMode::Fancy) {
            format!("{} {}", "❌".red(), msg.red().bold())
        } else {
            format!("✘ {}", msg)
        };
        pb.finish_with_message(final_msg);
    }
}

/// Convenience to finish spinner with success (compatibility)
pub fn finish_spinner(pb: ProgressBar, msg: &str) {
    finish_spinner_success(pb, msg);
}
