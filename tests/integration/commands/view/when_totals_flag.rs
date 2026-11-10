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
fn it_shows_totals() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 10:00 | Task one @done(2024-01-15 11:00)\n\t- 2024-01-15 12:00 | Task two @done(2024-01-15 13:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["view", "daily", "--totals"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Task one") || stdout.contains("Task two"),
    "expected entries in totals output, got: {stdout}"
  );
}
