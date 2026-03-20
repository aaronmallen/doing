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
