pub mod args;
pub mod commands;
pub mod editor;
pub mod interactive;
pub mod pager;

use std::{ffi::OsString, path::PathBuf};

use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use log::debug;
use yansi::Condition;

use crate::{
  config::{self, Config},
  errors::Result,
  taskpaper::{self, Document},
  template,
};

/// Shared application context passed to all command handlers.
#[allow(dead_code)]
pub(crate) struct AppContext {
  pub config: Config,
  pub default_answer: bool,
  pub document: Document,
  pub doing_file: PathBuf,
  pub include_notes: bool,
  pub no: bool,
  pub noauto: bool,
  pub quiet: bool,
  pub stdout: bool,
  pub use_color: bool,
  pub use_pager: bool,
  pub yes: bool,
}

impl AppContext {
  /// Print a user-facing status message to stderr.
  ///
  /// Respects `--quiet` — when quiet mode is active, the message is suppressed.
  pub fn status(&self, msg: impl std::fmt::Display) {
    if !self.quiet {
      eprintln!("{msg}");
    }
  }
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

  /// Specify a different doing_file [env: DOING_FILE=]
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

  #[arg(
    long = "no-pager",
    action = ArgAction::SetTrue,
    hide = true,
    overrides_with = "pager",
    global = true
  )]
  no_pager: bool,

  /// Exclude auto tags and default tags
  #[arg(short = 'X', long, action = ArgAction::SetTrue, overrides_with = "no_noauto", global = true)]
  noauto: bool,

  /// Output notes if included in the template
  #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_notes", global = true)]
  notes: bool,

  /// Use a pager when output is longer than screen
  #[arg(short = 'p', long, action = ArgAction::SetTrue, overrides_with = "no_pager", global = true)]
  pager: bool,

  /// Silence status messages
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
    template::colors::init();
    init_logger(self.log_level());

    debug!("CLI parsed successfully");

    let config = Config::load()?;
    let doing_file = self.doing_file.clone().unwrap_or_else(|| config.doing_file.clone());

    debug!("Using doing file: {}", doing_file.display());

    Document::create_file(&doing_file, &config.current_section)?;
    let document = taskpaper::io::read_file(&doing_file)?;

    let include_notes = if self.notes {
      true
    } else if self.no_notes {
      false
    } else {
      config.include_notes
    };

    let use_color = if self.color {
      true
    } else if self.no_color {
      false
    } else {
      Condition::stdouterr_are_tty_live()
    };

    if use_color {
      yansi::enable();
    } else {
      yansi::disable();
    }

    let use_pager = if self.pager {
      true
    } else if self.no_pager {
      false
    } else {
      config.paginate
    };

    let quiet = self.quiet || config::env::DOING_QUIET.value().unwrap_or(false);

    let mut ctx = AppContext {
      config,
      default_answer: self.default,
      document,
      doing_file,
      include_notes,
      no: self.no,
      noauto: self.noauto && !self.no_noauto,
      quiet,
      stdout: self.stdout,
      use_color,
      use_pager,
      yes: self.yes,
    };

    let default_cmd = Command::Recent(commands::recent::Command::default());
    self.command.as_ref().unwrap_or(&default_cmd).call(&mut ctx)
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
}

/// All available subcommands.
#[derive(Debug, Subcommand)]
enum Command {
  /// Repeat the last entry
  #[command(visible_alias = "resume")]
  Again(commands::again::Command),
  /// Move entries to the Archive section
  #[command(visible_alias = "move")]
  Archive(commands::archive::Command),
  /// Apply autotagging rules to existing entries
  Autotag(commands::autotag::Command),
  /// Manage time budgets for tags
  Budget(commands::budget::Command),
  /// Mark the last entry as cancelled
  Cancel(commands::cancel::Command),
  /// List recent changes in Doing
  Changes,
  /// Fuzzy select an entry to act on
  Choose(commands::choose::Command),
  /// Show available color template tokens
  Colors(commands::colors::Command),
  /// List available commands
  Commands(commands::commands::Command),
  /// List commands accepting a given option
  #[command(hide = true)]
  CommandsAccepting(commands::commands_accepting::Command),
  /// Generate shell completions
  Completion,
  /// View, edit, and manage configuration
  Config(commands::config::Command),
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
  Import(commands::import::Command),
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
  Open(commands::open::Command),
  /// List installed plugins
  Plugins(commands::plugins::Command),
  /// Show recent entries
  Recent(commands::recent::Command),
  /// Redo the last undone change
  Redo(commands::redo::Command),
  /// Reset the start date of the last entry
  #[command(visible_alias = "begin")]
  Reset(commands::reset::Command),
  /// Move entries to a dated archive file
  Rotate(commands::rotate::Command),
  /// List available sections
  Sections(commands::sections::Command),
  /// Interactively select entries to act on
  Select(commands::select::Command),
  /// Show entries from a section
  Show(commands::show::Command),
  /// Show entries since a given date
  Since(commands::since::Command),
  /// Add or remove tags from entries
  Tag(commands::tag::Command),
  /// Set the default tags directory
  TagDir(commands::tag_dir::Command),
  /// List all tags in the doing file
  Tags(commands::tags::Command),
  /// Show or edit entry templates
  Template(commands::template::Command),
  /// Show entries from today
  Today(commands::today::Command),
  /// Undo the last change
  Undo(commands::undo::Command),
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
      Self::Archive(cmd) => cmd.call(ctx),
      Self::Autotag(cmd) => cmd.call(ctx),
      Self::Budget(cmd) => cmd.call(ctx),
      Self::Cancel(cmd) => cmd.call(ctx),
      Self::Colors(cmd) => cmd.call(),
      Self::Commands(cmd) => cmd.call(&Cli::command()),
      Self::CommandsAccepting(cmd) => cmd.call(&Cli::command()),
      Self::Config(cmd) => cmd.call(ctx),
      Self::Done(cmd) => cmd.call(ctx),
      Self::External(args) => {
        let name = args.first().and_then(|s| s.to_str()).unwrap_or("");
        if ctx.config.views.contains_key(name) {
          commands::view::Command::call_external(name, ctx)
        } else if args.len() > 1 {
          let title: Vec<String> = args.iter().filter_map(|s| s.to_str().map(String::from)).collect();
          commands::now::Command::call_external(title, ctx)
        } else {
          Err(crate::errors::Error::Config(format!("unknown command: {name}")))
        }
      }
      Self::Finish(cmd) => cmd.call(ctx),
      Self::Grep(cmd) => cmd.call(ctx),
      Self::Import(cmd) => cmd.call(ctx),
      Self::Last(cmd) => cmd.call(ctx),
      Self::Mark(cmd) => cmd.call(ctx),
      Self::Meanwhile(cmd) => cmd.call(ctx),
      Self::Note(cmd) => cmd.call(ctx),
      Self::Now(cmd) => cmd.call(ctx),
      Self::On(cmd) => cmd.call(ctx),
      Self::Open(cmd) => cmd.call(ctx),
      Self::Plugins(cmd) => cmd.call(),
      Self::Recent(cmd) => cmd.call(ctx),
      Self::Redo(cmd) => cmd.call(ctx),
      Self::Reset(cmd) => cmd.call(ctx),
      Self::Rotate(cmd) => cmd.call(ctx),
      Self::Sections(cmd) => cmd.call(ctx),
      Self::Select(cmd) => cmd.call(ctx),
      Self::Show(cmd) => cmd.call(ctx),
      Self::Since(cmd) => cmd.call(ctx),
      Self::Tag(cmd) => cmd.call(ctx),
      Self::TagDir(cmd) => cmd.call(ctx),
      Self::Tags(cmd) => cmd.call(ctx),
      Self::Template(cmd) => cmd.call(ctx),
      Self::Today(cmd) => cmd.call(ctx),
      Self::Undo(cmd) => cmd.call(ctx),
      Self::View(cmd) => cmd.call(ctx),
      Self::Views(cmd) => cmd.call(ctx),
      Self::Yesterday(cmd) => cmd.call(ctx),
      Self::Changes => todo!(),
      Self::Choose(cmd) => cmd.call(ctx),
      Self::Completion => todo!(),
      Self::Update => todo!(),
    }
  }
}

pub fn run() -> Result<()> {
  Cli::parse().call()
}

fn init_logger(level: log::LevelFilter) {
  env_logger::Builder::new().filter_level(level).init();
}
