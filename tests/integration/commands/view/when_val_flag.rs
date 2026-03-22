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
#[ignore = "view --val flag query syntax needs investigation (see #185)"]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Client A work @project(clientA)"]).assert().success();
  doing.run(["now", "Client B work @project(clientB)"]).assert().success();

  let output = doing
    .run(["view", "daily", "--tag", "project", "--val", "clientA"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Client A work"),
    "expected Client A entry filtered by tag value, got: {stdout}"
  );
}
