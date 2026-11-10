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
fn it_respects_color_flag() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Color test entry"]).assert().success();

  // Use raw_cmd so --no-color isn't automatically added
  let mut cmd = doing.raw_cmd();
  cmd.args(["-f", doing.doing_file_path().to_str().unwrap()]);
  cmd.args(["view", "daily", "--color"]);

  let output = cmd.output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Color test entry"),
    "expected entry in colored output, got: {stdout}"
  );
}
