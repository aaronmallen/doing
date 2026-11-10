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
fn it_displays_configured_view() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "View test entry one"]).assert().success();
  doing.run(["now", "View test entry two"]).assert().success();

  let output = doing.run(["view", "daily"]).output().expect("failed to run");

  assert!(output.status.success(), "expected view command to succeed");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("View test entry one") || stdout.contains("View test entry two"),
    "expected view to show entries, got: {stdout}"
  );
}

#[test]
fn it_returns_error_for_unknown_view() {
  let doing = DoingCmd::new();

  let output = doing.run(["view", "nonexistent_view"]).output().expect("failed to run");

  assert!(!output.status.success(), "expected non-zero exit code for unknown view");
}
