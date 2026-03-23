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

const VIEW_CONFIG_WITH_INTERVAL: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title (%interval)%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"

[views.daily]
section = "Currently"
count = 10
order = "asc"
template = "%date | %title (%interval)"
"#;

#[test]
fn it_shows_duration_on_entries() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Duration test entry"]).assert().success();

  let output = doing
    .run(["view", "daily", "--duration"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Duration test entry"),
    "expected entry in duration output, got: {stdout}"
  );
}

#[test]
fn it_includes_interval_for_finished_entries() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG_WITH_INTERVAL);

  doing
    .run(["now", "--back", "1h", "Finished view entry"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["view", "daily", "--duration"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Finished view entry"),
    "expected entry in output, got: {stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    re.is_match(&stdout),
    "expected interval in HH:MM:SS format in output, got: {stdout}"
  );
}
