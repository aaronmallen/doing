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
fn it_filters_by_date_range() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Out of range entry\n\t- 2024-01-14 10:00 | In range entry\n\t- 2024-02-01 10:00 | Future entry\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["view", "daily", "--from", "2024-01-13 to 2024-01-16"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("In range entry"),
    "expected in-range entry with --from, got: {stdout}"
  );
}
