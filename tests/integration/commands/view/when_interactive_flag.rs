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
#[ignore = "interactive mode requires TTY and cannot be tested in CI"]
fn it_enables_interactive_mode() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Interactive test entry"]).assert().success();

  // Interactive mode requires a TTY, so this test is always ignored in CI
  let _output = doing
    .run(["view", "daily", "--interactive"])
    .output()
    .expect("failed to run");
}
