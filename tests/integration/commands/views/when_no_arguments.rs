use crate::support::helpers::DoingCmd;

const CONFIG_WITH_VIEWS: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[editors]
default = "cat"

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[views.done]
section = "All"
tags = "done complete cancelled"
tags_bool = "OR"

[views.color]
section = "Currently"
count = 10
"#;

#[test]
fn it_lists_all_view_names() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing.run(["views"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("done"), "expected 'done' view in output, got: {stdout}");
  assert!(
    stdout.contains("color"),
    "expected 'color' view in output, got: {stdout}"
  );
}

#[test]
fn it_shows_nothing_when_no_views() {
  let doing = DoingCmd::new();

  let output = doing.run(["views"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("No views") || stdout.trim().is_empty(),
    "expected empty or 'No views' message, got: {stdout}"
  );
}
