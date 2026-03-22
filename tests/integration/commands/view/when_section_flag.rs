use std::fs;

use crate::support::helpers::DoingCmd;

const VIEW_CONFIG: &str = r#"
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

[editors]
default = "cat"

[views.daily]
section = "Currently"
count = 10
order = "asc"
template = "%date | %title"
"#;

#[test]
fn it_overrides_view_section() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  // Pre-create doing file with two sections
  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 10:00 | Currently entry\nLater:\n\t- 2024-01-15 11:00 | Later entry\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["view", "daily", "--section", "Later"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Later entry"),
    "expected Later entry when overriding section, got: {stdout}"
  );
}

#[test]
fn it_overrides_with_short_flag() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 10:00 | Currently entry\nLater:\n\t- 2024-01-15 11:00 | Later short flag entry\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["view", "daily", "-s", "Later"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Later short flag entry"),
    "expected Later short flag entry with -s, got: {stdout}"
  );
}
