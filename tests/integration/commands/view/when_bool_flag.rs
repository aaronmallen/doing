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
fn it_uses_and_boolean() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Both tags @project @urgent"]).assert().success();
  doing.run(["now", "Only project @project"]).assert().success();
  doing.run(["now", "Only urgent @urgent"]).assert().success();

  let output = doing
    .run(["view", "daily", "--tag", "project,urgent", "--bool", "AND"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Both tags"),
    "expected entry with both tags in AND output, got: {stdout}"
  );
}

#[test]
fn it_uses_or_boolean() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Both tags @project @urgent"]).assert().success();
  doing.run(["now", "Only project @project"]).assert().success();
  doing.run(["now", "No tags entry"]).assert().success();

  let output = doing
    .run(["view", "daily", "--tag", "project,urgent", "--bool", "OR"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Both tags") || stdout.contains("Only project"),
    "expected entries with at least one tag in OR output, got: {stdout}"
  );
  assert!(
    !stdout.contains("No tags entry"),
    "expected entry without matching tags to be excluded, got: {stdout}"
  );
}
