use std::fs;

use assert_cmd::Command;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_uses_custom_doing_file_path() {
  let doing = DoingCmd::new();

  // The DoingCmd helper already uses -f to set a custom doing file path.
  // Verify that entries end up in the expected temp doing file.
  doing.run(["now", "Custom file entry"]).assert().success();

  let content = doing.read_doing_file();
  assert!(
    content.contains("Custom file entry"),
    "entry should be written to the doing file specified by -f flag"
  );
}

#[test]
fn it_suppresses_output_with_quiet_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["--quiet", "now", "Quiet entry"])
    .output()
    .expect("failed to run with --quiet");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(output.status.success(), "command should succeed with --quiet");
  assert!(stderr.is_empty(), "stderr should be empty with --quiet flag");
}

#[test]
fn it_disables_color_with_no_color_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Color test entry @tag1"]).assert().success();

  // The DoingCmd helper already passes --no-color, but let's verify output has no ANSI codes
  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    !stdout.contains("\x1b["),
    "output should not contain ANSI escape codes with --no-color"
  );
}

#[test]
fn it_disables_pager_with_no_pager_flag() {
  let doing = DoingCmd::new_with_config(
    r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = true

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"
"#,
  );

  // Create an entry
  doing.run(["now", "Test pager entry"]).assert().success();

  // Run show with --no-pager and verify output reaches stdout
  let output = doing.run(["--no-pager", "show"]).output().expect("failed to run show");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Test pager entry"),
    "expected entry in stdout when --no-pager is used"
  );
}

#[test]
fn it_skips_autotagging_with_noauto_flag() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

default_tags = ["tracked"]

[editors]
default = "cat"
"#;
  let doing = DoingCmd::new_with_config(config);

  doing
    .run(["--noauto", "now", "Entry without autotag"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    !content.contains("@tracked"),
    "default tags should not be applied with --noauto"
  );
}

#[test]
fn it_enables_debug_logging_with_debug_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["--debug", "now", "Debug entry"])
    .output()
    .expect("failed to run with --debug");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(output.status.success(), "command should succeed with --debug");
  // Debug mode should produce some log output on stderr
  assert!(
    !stderr.is_empty(),
    "stderr should contain debug output with --debug flag"
  );
}

#[test]
fn it_prefers_f_flag_over_doing_file_env_var() {
  let doing = DoingCmd::new();
  let temp = doing.temp_dir_path();
  let env_file = temp.join("env_doing.md");
  let flag_file = temp.join("flag_doing.md");
  let config_path = temp.join("config.toml");

  Command::cargo_bin("doing")
    .unwrap()
    .env("DOING_FILE", &env_file)
    .env("DOING_CONFIG", &config_path)
    .env("DOING_BACKUP_DIR", doing.backup_dir())
    .args(["-f", flag_file.to_str().unwrap(), "--no-color", "now", "flag entry"])
    .assert()
    .success();

  let flag_content = fs::read_to_string(&flag_file).expect("failed to read flag file");
  assert!(flag_content.contains("flag entry"), "entry should be in the -f file");

  let env_content = fs::read_to_string(&env_file).unwrap_or_default();
  assert!(
    !env_content.contains("flag entry"),
    "entry should not be in the DOING_FILE path"
  );
}

#[test]
fn it_sends_output_to_stdout_with_stdout_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Stdout entry"]).assert().success();

  let output = doing
    .run(["--stdout", "show"])
    .output()
    .expect("failed to run with --stdout");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "command should succeed with --stdout");
  assert_eq!(helpers::count_entries(&stdout), 1, "stdout should contain the entry");
}

#[test]
fn it_uses_doing_file_from_env_var() {
  let doing = DoingCmd::new();
  let temp = doing.temp_dir_path();
  let env_file = temp.join("env_doing.md");
  let config_path = temp.join("config.toml");

  // Create an entry using DOING_FILE env var (no -f flag)
  Command::cargo_bin("doing")
    .unwrap()
    .env("DOING_FILE", &env_file)
    .env("DOING_CONFIG", &config_path)
    .env("DOING_BACKUP_DIR", doing.backup_dir())
    .args(["--no-color", "now", "env entry"])
    .assert()
    .success();

  let content = fs::read_to_string(&env_file).expect("failed to read env file");
  assert!(content.contains("env entry"), "entry should be in the DOING_FILE path");

  // Verify show also reads from DOING_FILE
  let output = Command::cargo_bin("doing")
    .unwrap()
    .env("DOING_FILE", &env_file)
    .env("DOING_CONFIG", &config_path)
    .env("DOING_BACKUP_DIR", doing.backup_dir())
    .args(["--no-color", "show"])
    .output()
    .expect("failed to run show");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("env entry"),
    "show should display entry from DOING_FILE"
  );
}
