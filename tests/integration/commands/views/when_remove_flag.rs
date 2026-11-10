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
fn it_removes_view() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing
    .run(["views", "--remove", "done"])
    .output()
    .expect("failed to run");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(output.status.success(), "expected success exit code, stderr: {stderr}");

  // After removal, listing views should not show "done"
  let list_output = doing.run(["views"]).output().expect("failed to run");
  let stdout = String::from_utf8_lossy(&list_output.stdout);
  assert!(
    !stdout.contains("done"),
    "expected 'done' view to be removed, but still found in: {stdout}"
  );
}

#[test]
fn it_removes_with_short_flag() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing.run(["views", "-r", "color"]).output().expect("failed to run");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(output.status.success(), "expected success exit code, stderr: {stderr}");
}

#[test]
fn it_returns_error_for_nonexistent_view() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing
    .run(["views", "--remove", "nonexistent"])
    .output()
    .expect("failed to run");

  assert!(
    !output.status.success(),
    "expected non-zero exit code for removing nonexistent view"
  );
}
