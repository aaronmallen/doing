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
fn it_outputs_in_column_format() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing.run(["views", "--column"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  // In column format, each view name should be on its own line with no extra info
  assert!(
    lines.len() >= 2,
    "expected at least 2 view names in column output, got: {stdout}"
  );

  for line in &lines {
    let trimmed = line.trim();
    if !trimmed.is_empty() {
      assert!(
        trimmed == "done" || trimmed == "color",
        "expected only view names in column format, got: {trimmed}"
      );
    }
  }
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_VIEWS);

  let output = doing.run(["views", "-c"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(
    lines.len() >= 2,
    "expected at least 2 view names in column output, got: {stdout}"
  );
}
