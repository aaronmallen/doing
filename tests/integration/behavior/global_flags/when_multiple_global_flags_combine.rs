use crate::support::helpers::DoingCmd;

const COLOR_TEMPLATE_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%cyan%date | %title%reset%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

const AUTOTAG_CONFIG: &str = r#"
current_section = "Currently"
default_tags = ["tracked"]
doing_file_sort = "asc"
include_notes = true
paginate = false

[autotag]
whitelist = ["project"]
transform = []

[autotag.synonyms]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

#[test]
fn it_applies_debug_and_quiet_together() {
  let doing = DoingCmd::new();
  doing.run(["now", "Debug quiet combo"]).assert().success();

  let output = doing
    .run(["--debug", "--quiet", "show"])
    .output()
    .expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(output.status.success(), "expected success with --debug and --quiet");

  // Quiet takes precedence over debug — debug output is suppressed
  assert!(
    stderr.is_empty(),
    "expected empty stderr when --quiet overrides --debug, got: {stderr}"
  );
}

#[test]
fn it_applies_noauto_and_doing_file_together() {
  let doing = DoingCmd::new_with_config(AUTOTAG_CONFIG);
  let custom = doing.temp_dir_path().join("combo.md");

  doing
    .raw_cmd()
    .arg("--no-color")
    .arg("-f")
    .arg(&custom)
    .arg("now")
    .arg("-X")
    .arg("Combined noauto file test")
    .assert()
    .success();

  let contents = std::fs::read_to_string(&custom).expect("custom file should exist");
  assert!(
    contents.contains("Combined noauto file test"),
    "expected entry in custom file, got: {contents}"
  );
  assert!(
    !contents.contains("@tracked"),
    "expected no @tracked tag with -X and custom file, got: {contents}"
  );
}

#[test]
fn it_applies_quiet_and_no_color_together() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);

  let output = doing
    .run(["--quiet", "now", "Quiet no-color combo"])
    .output()
    .expect("failed to run doing");
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(stderr.is_empty(), "expected empty stderr with --quiet, got: {stderr}");

  let output = doing.run(["show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // DoingCmd always adds --no-color, so no ANSI codes
  assert!(
    !stdout.contains("\x1b["),
    "expected no ANSI codes with --no-color, got: {stdout}"
  );
}

#[ignore = "--stdout flag does not yet redirect status messages to stdout (#155)"]
#[test]
fn it_applies_stdout_and_no_color_together() {
  let doing = DoingCmd::new_with_config(COLOR_TEMPLATE_CONFIG);
  doing
    .raw_cmd()
    .arg("-f")
    .arg(doing.doing_file_path())
    .arg("--no-color")
    .arg("now")
    .arg("Stdout no-color combo")
    .output()
    .expect("failed to run doing");

  let output = doing
    .raw_cmd()
    .arg("-f")
    .arg(doing.doing_file_path())
    .arg("--stdout")
    .arg("--no-color")
    .arg("show")
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    stdout.contains("Stdout no-color combo"),
    "expected entry on stdout, got: {stdout}"
  );
  assert!(
    !stdout.contains("\x1b["),
    "expected no ANSI codes with --no-color, got: {stdout}"
  );
  assert!(stderr.is_empty(), "expected empty stderr with --stdout, got: {stderr}");
}

#[ignore = "--stdout flag does not yet redirect status messages to stdout (#155)"]
#[test]
fn it_applies_stdout_and_quiet_together() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["--stdout", "--quiet", "now", "Stdout quiet combo"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  // Both stdout and stderr should be empty: quiet suppresses, stdout redirects
  assert!(
    stdout.is_empty(),
    "expected empty stdout with --stdout --quiet, got: {stdout}"
  );
  assert!(
    stderr.is_empty(),
    "expected empty stderr with --stdout --quiet, got: {stderr}"
  );
}
