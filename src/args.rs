use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gitpulse",
    version,
    about = "Pulse-check every local git repo: dirty trees, unpushed commits, stale branches, missing remotes",
    long_about = None
)]
pub struct Args {
    /// Root directory to scan for repositories
    #[arg(default_value = "~/Projects")]
    pub root: PathBuf,

    /// How deep to descend while looking for repositories
    #[arg(long, default_value_t = 4)]
    pub max_depth: usize,

    /// Also descend into hidden directories
    #[arg(short = 'H', long)]
    pub hidden: bool,

    /// Include clean repositories in the report
    #[arg(short, long)]
    pub all: bool,

    /// A commit or branch tip older than this many days counts as stale
    #[arg(long, default_value_t = 90)]
    pub stale_days: u64,

    /// Emit JSON instead of a table
    #[arg(long)]
    pub json: bool,

    /// When to use terminal colors
    #[arg(long, value_enum, default_value_t = Color::Auto)]
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Color {
    Auto,
    Always,
    Never,
}

impl Color {
    pub fn enabled(self) -> bool {
        match self {
            Color::Always => true,
            Color::Never => false,
            Color::Auto => {
                std::io::IsTerminal::is_terminal(&std::io::stdout())
                    && std::env::var_os("NO_COLOR").is_none()
            }
        }
    }
}
