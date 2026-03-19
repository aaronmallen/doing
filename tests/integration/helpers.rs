use std::{ffi::OsStr, fs, path::PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// Entry line pattern: `2024-01-15 14:30 | Some entry text`
const ENTRY_PATTERN: &str = r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2} \|";

/// Minimal TOML config for isolated tests.
const TEST_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

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
  _temp_dir: TempDir,
  config_path: PathBuf,
  doing_file_path: PathBuf,
}

impl DoingCmd {
  /// Create a new isolated test environment.
  pub fn new() -> Self {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let doing_file_path = temp_dir.path().join("doing.md");
    let config_path = temp_dir.path().join("config.toml");

    // Write the config with doing_file pointing into the temp dir
    let config_content = format!("{TEST_CONFIG}\n[editors]\ndefault = \"cat\"\n",);
    fs::write(&config_path, config_content).expect("failed to write test config");

    Self {
      _temp_dir: temp_dir,
      config_path,
      doing_file_path,
    }
  }

  /// Build an `assert_cmd::Command` for the `doing` binary with isolation env vars set.
  pub fn cmd(&self) -> Command {
    let mut cmd = Command::cargo_bin("doing").expect("failed to find doing binary");
    cmd.env("DOING_CONFIG", &self.config_path);
    cmd.arg("-f").arg(&self.doing_file_path);
    cmd.arg("--no-color");
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
}

/// Count the number of entry lines in output matching the standard entry pattern.
pub fn count_entries(output: &str) -> usize {
  let re = regex::Regex::new(ENTRY_PATTERN).expect("invalid entry regex");
  output.lines().filter(|line| re.is_match(line)).count()
}
