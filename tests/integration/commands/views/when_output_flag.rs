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
fn it_outputs_in_specified_format() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing
    .run(["views", "--output", "json", "done"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  // JSON output should be parseable
  assert!(
    stdout.contains("{") || stdout.contains("["),
    "expected JSON output, got: {stdout}"
  );
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing
    .run(["views", "-o", "json", "done"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("{") || stdout.contains("["),
    "expected JSON output with short flag, got: {stdout}"
  );
}
