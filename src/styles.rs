//! Styling helpers for colored output and spinners.
//!
//! Centralizes color usage and effects so the rest of the code can
//! print consistently. Use `Styles::init(opts)` once at startup, then
//! use the helpers like `ok`, `warn`, `err`, and `spinner`.

use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::OnceCell;
use owo_colors::OwoColorize;

/// Controls how CLI renders output
#[derive(Clone, Copy, Debug)]
pub enum RenderMode {
    /// Colors and spinners enabled
    Fancy,
    /// Minimal: no color, no spinner
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

pub fn ok<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color {
        println!("{} {}", "✔".green().bold(), m.green());
    } else {
        println!("[OK] {}", m);
    }
}

pub fn info<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color {
        println!("{} {}", "ℹ".blue().bold(), m);
    } else {
        println!("[INFO] {}", m);
    }
}

pub fn warn<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color {
        eprintln!("{} {}", "⚠".yellow().bold(), m.yellow());
    } else {
        eprintln!("[WARN] {}", m);
    }
}

#[allow(dead_code)]
pub fn err<S: AsRef<str>>(msg: S) {
    if opts().quiet {
        return;
    }
    let m = msg.as_ref();
    if opts().use_color {
        eprintln!("{} {}", "✘".red().bold(), m.red());
    } else {
        eprintln!("[ERR] {}", m);
    }
}

/// Create a spinner if Fancy mode; otherwise returns a no-op spinner.
pub fn spinner<S: AsRef<str>>(msg: S) -> ProgressBar {
    let o = opts();
    if matches!(o.mode, RenderMode::Fancy) && !o.quiet {
        let pb = ProgressBar::new_spinner();
        let sty = if o.use_color {
            ProgressStyle::with_template("{spinner:.green} {wide_msg}").unwrap()
        } else {
            ProgressStyle::default_spinner()
        };
        pb.set_style(sty);
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb.set_message(msg.as_ref().to_string());
        pb
    } else {
        ProgressBar::hidden()
    }
}

/// Convenience to finish spinner with success.
pub fn finish_spinner(pb: ProgressBar, msg: &str) {
    if pb.is_hidden() {
        info(msg);
    } else {
        pb.finish_with_message(msg.to_string());
    }
}
