use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_formats_totals_as_clock() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @project @done(2024-01-15 10:30)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--totals-format", "clock"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("01:30:00"),
    "expected clock format in totals, got: {stdout}"
  );
}

#[test]
fn it_formats_totals_as_natural() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @project @done(2024-01-15 10:30)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--totals-format", "natural"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("about an hour and a half"),
    "expected natural format in totals, got: {stdout}"
  );
}

#[test]
fn it_formats_totals_as_hm() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @project @done(2024-01-15 10:30)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--totals", "--totals-format", "hm"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("01:30"), "expected hm format in totals, got: {stdout}");
}

#[test]
fn it_respects_config_totals_format() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false
totals_format = "natural"

[interaction]
confirm_longer_than = ""

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;
  let doing = DoingCmd::new_with_config(config);

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Task A @project @done(2024-01-15 10:30)\n",
  )
  .expect("failed to write doing file");

  let output = doing.run(["show", "--totals"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("about an hour and a half"),
    "expected natural format from config, got: {stdout}"
  );
}
