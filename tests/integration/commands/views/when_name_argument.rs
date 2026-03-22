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
"#;

#[test]
#[ignore = "our views command does not support positional name argument to dump config (see #189)"]
fn it_dumps_view_config() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing.run(["views", "done"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("done") && (stdout.contains("section") || stdout.contains("All")),
    "expected view config output for 'done', got: {stdout}"
  );
}

#[test]
fn it_returns_error_for_unknown_view() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing.run(["views", "nonexistent"]).output().expect("failed to run");

  assert!(!output.status.success(), "expected non-zero exit code for unknown view");
}
