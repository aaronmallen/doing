pub mod args;

use std::{ffi::OsString, path::PathBuf};

use clap::{ArgAction, Parser, Subcommand};
use log::debug;

use crate::{
  config::{self, Config},
  errors::Result,
  taskpaper::{self, Document},
};

/// Shared application context passed to all command handlers.
struct AppContext {
  pub config: Config,
  pub document: Document,
  pub doing_file: PathBuf,
}

/// A CLI for a What Was I Doing system.
#[derive(Debug, Parser)]
#[command(
  about = env!("CARGO_PKG_DESCRIPTION"),
  author = "Aaron Allen <@aaronmallen>",
  long_about = "Doing uses a TaskPaper-like formatting to keep a plain text record of what \
    you've been doing, complete with tag-based time tracking. The command line tool allows you \
    to add entries, annotate with tags and notes, and view your entries with myriad options, \
    with a focus on a \"natural\" language syntax.",
  name = "doing",
  version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
  /// Colored output
  #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_color", global = true)]
  color: bool,

  #[command(subcommand)]
  command: Option<Command>,

  /// Enable debug output
  #[arg(long, global = true)]
  debug: bool,

  /// Answer yes/no menus with default option
  #[arg(long, global = true)]
  default: bool,

  /// Specify a different doing_file
  #[arg(short = 'f', long, global = true)]
  doing_file: Option<PathBuf>,

  /// Answer all yes/no menus with no
  #[arg(long, global = true)]
  no: bool,

  #[arg(long = "no-color", action = ArgAction::SetTrue, hide = true, overrides_with = "color", global = true)]
  no_color: bool,

  #[arg(
    long = "no-noauto",
    action = ArgAction::SetTrue,
    hide = true,
    overrides_with = "noauto",
    global = true
  )]
  no_noauto: bool,

  #[arg(
    long = "no-notes",
    action = ArgAction::SetTrue,
    hide = true,
    overrides_with = "notes",
    global = true
  )]
  no_notes: bool,

  /// Exclude auto tags and default tags
  #[arg(short = 'x', long, action = ArgAction::SetTrue, overrides_with = "no_noauto", global = true)]
  noauto: bool,

  /// Output notes if included in the template
  #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_notes", global = true)]
  notes: bool,

  /// Use a pager when output is longer than screen
  #[arg(short = 'p', long, global = true)]
  pager: bool,

  /// Silence info messages
  #[arg(short = 'q', long, global = true)]
  quiet: bool,

  /// Send results report to STDOUT instead of STDERR
  #[arg(long, global = true)]
  stdout: bool,

  /// Answer all yes/no menus with yes
  #[arg(long, global = true)]
  yes: bool,
}

impl Cli {
  fn call(&self) -> Result<()> {
    init_logger(self.log_level());

    debug!("CLI parsed successfully");

    let config = Config::load()?;
    let doing_file = self.doing_file.clone().unwrap_or_else(|| config.doing_file.clone());

    debug!("Using doing file: {}", doing_file.display());

    Document::create_file(&doing_file, &config.current_section)?;
    let document = taskpaper::io::read_file(&doing_file)?;

    let ctx = AppContext {
      config,
      document,
      doing_file,
    };

    self.command.as_ref().unwrap_or(&Command::Recent).call(&ctx)
  }

  fn include_notes(&self) -> bool {
    !self.no_notes
  }

  fn log_level(&self) -> log::LevelFilter {
    let env_debug = config::env::DOING_DEBUG.value().unwrap_or(false);
    let env_level = config::env::DOING_LOG_LEVEL.value().ok();
    let env_quiet = config::env::DOING_QUIET.value().unwrap_or(false);

    // Quiet always wins, then debug, then env level, then default (info)
    if self.quiet || env_quiet {
      log::LevelFilter::Error
    } else if self.debug || env_debug {
      log::LevelFilter::Debug
    } else if let Some(ref level) = env_level {
      match level.to_lowercase().as_str() {
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        "info" => log::LevelFilter::Info,
        "trace" => log::LevelFilter::Trace,
        "warn" => log::LevelFilter::Warn,
        _ => log::LevelFilter::Info,
      }
    } else {
      log::LevelFilter::Info
    }
  }

  fn use_color(&self) -> bool {
    !self.no_color
  }
}

/// All available subcommands.
#[derive(Debug, Subcommand)]
enum Command {
  /// Repeat the last entry
  Again,
  /// Move entries to the Archive section
  Archive,
  /// Mark the last entry as cancelled
  Cancel,
  /// Show changes to the doing file
  Changes,
  /// Fuzzy select an entry to act on
  Choose,
  /// Show available color template tokens
  Colors,
  /// List available commands
  Commands,
  /// List commands accepting a given option
  #[command(name = "commands_accepting", hide = true)]
  CommandsAccepting,
  /// Generate shell completions
  Completion,
  /// Edit the configuration file
  Config,
  /// Add a completed entry
  Done,
  /// Catch-all for unknown subcommands (checked against custom views)
  #[command(external_subcommand)]
  External(Vec<OsString>),
  /// Mark the last entry as finished
  Finish,
  /// Flag the last entry
  Flag,
  /// Search for entries matching a pattern
  Grep,
  /// Import entries from other sources
  Import,
  /// Install fzf for fuzzy selection
  #[command(name = "install_fzf", hide = true)]
  InstallFzf,
  /// Show the last entry
  Last,
  /// Add an entry while finishing the last
  Meanwhile,
  /// Add or display notes on the last entry
  Note,
  /// Add a new entry
  Now,
  /// Show entries from a specific date
  On,
  /// Open the doing file in an editor
  Open,
  /// List installed plugins
  Plugins,
  /// Show recent entries
  Recent,
  /// Redo the last undone change
  Redo,
  /// Reset the doing file
  Reset,
  /// Move entries between sections
  Rotate,
  /// List available sections
  Sections,
  /// Interactively select entries to act on
  Select,
  /// Show entries from a section
  Show,
  /// Show entries since a given date
  Since,
  /// Add or remove tags from entries
  Tag,
  /// Set the default tags directory
  #[command(name = "tag_dir")]
  TagDir,
  /// List all tags in the doing file
  Tags,
  /// Show or edit entry templates
  Template,
  /// Show entries from today
  Today,
  /// Undo the last change
  Undo,
  /// Update doing to the latest version
  #[command(hide = true)]
  Update,
  /// Display a custom view
  View,
  /// List available views
  Views,
  /// Show entries from yesterday
  Yesterday,
}

impl Command {
  fn call(&self, _ctx: &AppContext) -> Result<()> {
    todo!()
  }
}

pub fn run() -> Result<()> {
  Cli::parse().call()
}

fn init_logger(level: log::LevelFilter) {
  env_logger::Builder::new().filter_level(level).init();
}
