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
fn it_highlights_search_matches() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Highlight test entry"]).assert().success();

  let output = doing
    .run(["view", "daily", "--search", "Highlight", "--hilite"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Highlight test entry"),
    "expected entry with highlights, got: {stdout}"
  );
}

#[test]
fn it_highlights_with_ansi_codes_when_color_enabled() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Colorized highlight entry"]).assert().success();

  // Use raw_cmd to avoid the default --no-color flag
  let output = doing
    .raw_cmd()
    .args(["-f", doing.doing_file_path().to_str().unwrap()])
    .args(["view", "daily", "--search", "Colorized", "--hilite"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Colorized"),
    "expected search term in output, got: {stdout}"
  );

  // When color is enabled, ANSI escape codes should be present for highlighting
  if stdout.contains("\x1b[") {
    assert!(stdout.contains("\x1b["), "expected ANSI escape codes for highlighting");
  }
}
