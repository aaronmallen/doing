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
#[ignore = "view --output json does not produce JSON output (see #206)"]
fn it_overrides_output_format() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "JSON output entry"]).assert().success();

  let output = doing
    .run(["view", "daily", "--output", "json"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // JSON output should contain JSON structure
  assert!(
    stdout.contains("\"title\"") || stdout.contains("\"section\"") || stdout.contains("["),
    "expected JSON output format, got: {stdout}"
  );
}

#[test]
#[ignore = "view -o json does not produce JSON output (see #206)"]
fn it_overrides_with_short_flag() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Short output entry"]).assert().success();

  let output = doing
    .run(["view", "daily", "-o", "json"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("\"title\"") || stdout.contains("\"section\"") || stdout.contains("["),
    "expected JSON output format with -o, got: {stdout}"
  );
}
