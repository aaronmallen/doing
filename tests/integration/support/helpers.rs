use std::{ffi::OsStr, fs, path::PathBuf};

use assert_cmd::Command;
use chrono::Local;
use regex::Regex;
use tempfile::TempDir;

/// Entry line pattern: matches the built-in default template (`  9:01am ║ text`).
const BUILTIN_ENTRY_PATTERN: &str = r"^\s*\S+\s+║";
/// Entry line pattern: matches explicit template (`2024-01-15 14:30 | text`).
const ENTRY_PATTERN: &str = r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2} \|";

/// Minimal TOML config for isolated tests.
const TEST_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"
"#;

/// Builder for running the `doing` binary in an isolated temp environment.
///
/// Each instance creates its own temp directory with a config file pointing
/// `doing_file` to that directory, ensuring complete test isolation.
pub struct DoingCmd {
  backup_dir: PathBuf,
  config_path: PathBuf,
  doing_file_path: PathBuf,
  _temp_dir: TempDir,
}

impl DoingCmd {
  /// Create a new isolated test environment.
  pub fn new() -> Self {
    let config_content = format!("{TEST_CONFIG}\n[editors]\ndefault = \"cat\"\n",);
    Self::with_config_content(&config_content)
  }

  /// Create a new isolated test environment with a custom config string.
  pub fn new_with_config(config: &str) -> Self {
    Self::with_config_content(config)
  }

  fn with_config_content(config_content: &str) -> Self {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let backup_dir = temp_dir.path().join("doing_backup");
    let doing_file_path = temp_dir.path().join("doing.md");
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, config_content).expect("failed to write test config");

    Self {
      _temp_dir: temp_dir,
      backup_dir,
      config_path,
      doing_file_path,
    }
  }

  /// Build an `assert_cmd::Command` for the `doing` binary with isolation env vars set.
  pub fn cmd(&self) -> Command {
    let mut cmd = Command::cargo_bin("doing").expect("failed to find doing binary");
    cmd.env("DOING_BACKUP_DIR", &self.backup_dir);
    cmd.env("DOING_CONFIG", &self.config_path);
    cmd.arg("-f").arg(&self.doing_file_path);
    cmd.arg("--no-color");
    cmd
  }

  /// Return the path to the doing file for this test environment.
  pub fn doing_file_path(&self) -> &std::path::Path {
    &self.doing_file_path
  }

  /// Build an `assert_cmd::Command` with only env-var isolation (no `-f` or `--no-color`).
  ///
  /// Use this when testing global flags that conflict with the defaults added by [`cmd`].
  pub fn raw_cmd(&self) -> Command {
    let mut cmd = Command::cargo_bin("doing").expect("failed to find doing binary");
    cmd.env("DOING_BACKUP_DIR", &self.backup_dir);
    cmd.env("DOING_CONFIG", &self.config_path);
    cmd
  }

  /// Read the contents of the doing file after commands have run.
  pub fn read_doing_file(&self) -> String {
    fs::read_to_string(&self.doing_file_path).unwrap_or_default()
  }

  /// Run a doing subcommand with the given arguments and return the command for assertions.
  pub fn run<I, S>(&self, args: I) -> Command
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    let mut cmd = self.cmd();
    cmd.args(args);
    cmd
  }

  /// Return the path to the temporary directory backing this test environment.
  pub fn temp_dir_path(&self) -> &std::path::Path {
    self._temp_dir.path()
  }
}

/// Assert two timestamp strings are within `tolerance` minutes of each other.
pub fn assert_times_within_tolerance(actual: &str, expected: &str, tolerance_minutes: i64, context: &str) {
  let fmt = "%Y-%m-%d %H:%M";
  let actual_dt = chrono::NaiveDateTime::parse_from_str(actual, fmt)
    .unwrap_or_else(|_| panic!("failed to parse actual time: {actual}"));
  let expected_dt = chrono::NaiveDateTime::parse_from_str(expected, fmt)
    .unwrap_or_else(|_| panic!("failed to parse expected time: {expected}"));
  let diff = (actual_dt - expected_dt).num_minutes().abs();
  assert!(
    diff <= tolerance_minutes,
    "{context}: expected {expected}, got {actual} (diff {diff} minutes, tolerance {tolerance_minutes})"
  );
}

/// Count the number of entry lines in output matching either the explicit template
/// pattern (`YYYY-MM-DD HH:MM |`) or the built-in default pattern (`time ║`).
pub fn count_entries(output: &str) -> usize {
  let re = regex::Regex::new(ENTRY_PATTERN).expect("invalid entry regex");
  let builtin_re = regex::Regex::new(BUILTIN_ENTRY_PATTERN).expect("invalid builtin entry regex");
  output
    .lines()
    .filter(|line| re.is_match(line) || builtin_re.is_match(line))
    .count()
}

/// Extract the @done(timestamp) value from the doing file
pub fn extract_done_timestamp(contents: &str) -> String {
  let re = Regex::new(r"@done\((\d{4}-\d{2}-\d{2} \d{2}:\d{2})\)").expect("invalid done timestamp regex");
  let cap = re
    .captures(contents)
    .expect("doing file should contain a @done timestamp");
  cap[1].to_string()
}

/// Extract the entry start timestamp from the doing file (e.g., `2024-03-17 14:30`)
pub fn extract_entry_timestamp(contents: &str) -> String {
  let re = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}) \|").expect("invalid entry timestamp regex");
  let cap = re
    .captures(contents)
    .expect("doing file should contain an entry timestamp");
  cap[1].to_string()
}

/// Format a chrono DateTime to the timestamp format used by doing entries.
pub fn fmt_time(dt: chrono::DateTime<Local>) -> String {
  dt.format("%Y-%m-%d %H:%M").to_string()
}
