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
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Meeting with team"]).assert().success();
  doing.run(["now", "meeting with client"]).assert().success();

  let output = doing
    .run(["view", "daily", "--search", "Meeting", "--case", "sensitive"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Meeting with team"),
    "expected 'Meeting with team' (capital M) in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("meeting with client"),
    "expected 'meeting with client' (lowercase) to be excluded, got: {stdout}"
  );
}
