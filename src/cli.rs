use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "DevProjectsCleaner",
    version,
    about = "Clean temporary and build files from any project",
    long_about = "Scans a project directory for temporary files, build artifacts, \
                   virtual environments, and caches across multiple programming \
                   languages.\n\n\
                   Interactive by default: the path is asked first, then a simple \
                   hierarchy of what was found is shown and confirmation is required \
                   before touching anything. Items are moved to the recycle bin \
                   (SAFE MODE); use --force to permanently delete them instead.\n\n\
                   Patterns are parameterizable via a TOML config file and the \
                   --add / --ignore flags."
)]
pub struct CliArgs {
    /// Project directory to clean (prompted interactively if omitted)
    #[arg(help = "Project directory to clean (prompted if omitted)")]
    pub path: Option<PathBuf>,

    /// Permanently delete instead of moving to trash
    #[arg(
        short = 'f',
        long = "force",
        help = "Permanently delete (skips SAFE MODE)"
    )]
    pub force: bool,

    /// Show the full list of every item instead of the hierarchy
    #[arg(short = 'v', long = "verbose", help = "List every single item found")]
    pub verbose: bool,

    /// Preview what would be cleaned without touching anything
    #[arg(short = 'n', long = "dry-run", help = "Preview items without deleting")]
    pub dry_run: bool,

    /// Skip all confirmation prompts
    #[arg(short = 'y', long = "yes", help = "Skip all confirmation prompts")]
    pub yes: bool,

    /// Add an extra pattern to clean (may be repeated)
    #[arg(long = "add", help = "Add an extra pattern to clean (repeatable)")]
    pub add: Vec<String>,

    /// Ignore a name so it is never cleaned (may be repeated)
    #[arg(long = "ignore", help = "Never clean a name (repeatable)")]
    pub ignore: Vec<String>,

    /// Path to a dev-projects-cleaner.toml config file
    #[arg(
        long = "config",
        help = "Use a custom dev-projects-cleaner.toml config file"
    )]
    pub config: Option<PathBuf>,

    /// Write the run log to a custom path
    #[arg(long = "log", help = "Write the run log to a custom path")]
    pub log: Option<PathBuf>,

    /// Do not write a log file
    #[arg(long = "no-log", help = "Do not write a log file")]
    pub no_log: bool,
}
