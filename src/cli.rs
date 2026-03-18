pub mod args;
pub mod commands;
pub mod editor;
pub mod pager;

use std::{ffi::OsString, path::PathBuf};

use clap::{ArgAction, Parser, Subcommand};
use log::debug;

use crate::{
  config::{self, Config},
  errors::Result,
  taskpaper::{self, Document},
};

/// Shared application context passed to all command handlers.
pub(crate) struct AppContext {
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

    let mut ctx = AppContext {
      config,
      document,
      doing_file,
    };

    let default_cmd = Command::Recent(commands::recent::Command::default());
    self.command.as_ref().unwrap_or(&default_cmd).call(&mut ctx)
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
  #[command(visible_alias = "resume")]
  Again(commands::again::Command),
  /// Apply autotagging rules to existing entries
  Autotag(commands::autotag::Command),
  /// Move entries to the Archive section
  Archive,
  /// Mark the last entry as cancelled
  Cancel(commands::cancel::Command),
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
  #[command(visible_alias = "did")]
  Done(commands::done::Command),
  /// Catch-all for unknown subcommands (checked against custom views)
  #[command(external_subcommand)]
  External(Vec<OsString>),
  /// Mark the last entry as finished
  Finish(commands::finish::Command),
  /// Search for entries matching a pattern
  #[command(visible_alias = "search")]
  Grep(commands::grep::Command),
  /// Import entries from other sources
  Import,
  /// Install fzf for fuzzy selection
  #[command(name = "install_fzf", hide = true)]
  InstallFzf,
  /// Show the last entry
  Last(commands::last::Command),
  /// Toggle the marker tag on the last entry
  #[command(visible_alias = "flag")]
  Mark(commands::mark::Command),
  /// Add an entry while finishing the last
  Meanwhile(commands::meanwhile::Command),
  /// Add or display notes on the last entry
  Note(commands::note::Command),
  /// Add a new entry
  #[command(visible_alias = "next")]
  Now(commands::now::Command),
  /// Show entries from a specific date
  On(commands::on::Command),
  /// Open the doing file in an editor
  Open,
  /// List installed plugins
  Plugins,
  /// Show recent entries
  Recent(commands::recent::Command),
  /// Redo the last undone change
  Redo,
  /// Reset the start date of the last entry
  #[command(visible_alias = "begin")]
  Reset(commands::reset::Command),
  /// Move entries between sections
  Rotate,
  /// List available sections
  Sections(commands::sections::Command),
  /// Interactively select entries to act on
  Select,
  /// Show entries from a section
  Show(commands::show::Command),
  /// Show entries since a given date
  Since(commands::since::Command),
  /// Add or remove tags from entries
  Tag(commands::tag::Command),
  /// Set the default tags directory
  #[command(name = "tag_dir")]
  TagDir,
  /// List all tags in the doing file
  Tags,
  /// Show or edit entry templates
  Template,
  /// Show entries from today
  Today(commands::today::Command),
  /// Undo the last change
  Undo,
  /// Update doing to the latest version
  #[command(hide = true)]
  Update,
  /// Display a custom view
  View(commands::view::Command),
  /// List available views
  Views(commands::views::Command),
  /// Show entries from yesterday
  Yesterday(commands::yesterday::Command),
}

impl Command {
  fn call(&self, ctx: &mut AppContext) -> Result<()> {
    match self {
      Self::Again(cmd) => cmd.call(ctx),
      Self::Autotag(cmd) => cmd.call(ctx),
      Self::Cancel(cmd) => cmd.call(ctx),
      Self::Done(cmd) => cmd.call(ctx),
      Self::Finish(cmd) => cmd.call(ctx),
      Self::Grep(cmd) => cmd.call(ctx),
      Self::Last(cmd) => cmd.call(ctx),
      Self::Mark(cmd) => cmd.call(ctx),
      Self::Meanwhile(cmd) => cmd.call(ctx),
      Self::Note(cmd) => cmd.call(ctx),
      Self::Now(cmd) => cmd.call(ctx),
      Self::On(cmd) => cmd.call(ctx),
      Self::Recent(cmd) => cmd.call(ctx),
      Self::Reset(cmd) => cmd.call(ctx),
      Self::Sections(cmd) => cmd.call(ctx),
      Self::Show(cmd) => cmd.call(ctx),
      Self::Since(cmd) => cmd.call(ctx),
      Self::Tag(cmd) => cmd.call(ctx),
      Self::Today(cmd) => cmd.call(ctx),
      Self::View(cmd) => cmd.call(ctx),
      Self::Views(cmd) => cmd.call(ctx),
      Self::Yesterday(cmd) => cmd.call(ctx),
      Self::External(args) => {
        let name = args.first().and_then(|s| s.to_str()).unwrap_or("");
        if ctx.config.views.contains_key(name) {
          commands::view::Command::call_external(name, ctx)
        } else {
          Err(crate::errors::Error::Config(format!("unknown command: {name}")))
        }
      }
      _ => todo!(),
    }
  }
}

pub fn run() -> Result<()> {
  Cli::parse().call()
}

fn init_logger(level: log::LevelFilter) {
  env_logger::Builder::new().filter_level(level).init();
}
