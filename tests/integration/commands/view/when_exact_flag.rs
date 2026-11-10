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
fn it_uses_exact_matching() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Meeting with team"]).assert().success();
  doing.run(["now", "Team meeting notes"]).assert().success();

  let output = doing
    .run(["view", "daily", "--search", "Meeting with team", "--exact"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Meeting with team"),
    "expected exact match 'Meeting with team' in output, got: {stdout}"
  );
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Meeting with team"]).assert().success();
  doing.run(["now", "Team meeting notes"]).assert().success();

  let output = doing
    .run(["view", "daily", "--search", "Meeting with team", "-x"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Meeting with team"),
    "expected exact match 'Meeting with team' with -x, got: {stdout}"
  );
}
