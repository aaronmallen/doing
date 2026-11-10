use crate::support::helpers::{DoingCmd, count_entries};

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
fn it_overrides_view_count() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Count entry one"]).assert().success();
  doing.run(["now", "Count entry two"]).assert().success();
  doing.run(["now", "Count entry three"]).assert().success();

  let output = doing
    .run(["view", "daily", "--count", "2"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_count = count_entries(&stdout);
  assert!(
    entry_count <= 2,
    "expected at most 2 entries with --count 2, got {entry_count}: {stdout}"
  );
}

#[test]
fn it_overrides_with_short_flag() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Short count entry one"]).assert().success();
  doing.run(["now", "Short count entry two"]).assert().success();
  doing.run(["now", "Short count entry three"]).assert().success();

  let output = doing.run(["view", "daily", "-c", "1"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_count = count_entries(&stdout);
  assert!(
    entry_count <= 1,
    "expected at most 1 entry with -c 1, got {entry_count}: {stdout}"
  );
}
