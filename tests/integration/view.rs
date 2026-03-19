use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

const VIEW_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[views.test]
section = "Currently"
count = 10
template = "default"
order = "asc"

[views.done]
section = "Done"
count = 5
template = "default"
order = "desc"
tags = "done"
tags_bool = "OR"

[editors]
default = "cat"
"#;

#[test]
fn it_lists_all_configured_views() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  let output = doing.run(["views"]).output().expect("failed to run views");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("test"), "views output should list 'test' view");
  assert!(stdout.contains("done"), "views output should list 'done' view");
}

#[test]
fn it_renders_named_view_with_settings() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Test entry one"]).assert().success();
  doing.run(["now", "Test entry two"]).assert().success();

  let output = doing.run(["view", "test"]).output().expect("failed to run view");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "view 'test' should render entries from Currently section"
  );
  assert!(stdout.contains("Test entry one"), "view should include first entry");
  assert!(stdout.contains("Test entry two"), "view should include second entry");
}

#[test]
fn it_respects_view_count_setting() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  for i in 1..=8 {
    doing.run(["now", &format!("Entry number {i}")]).assert().success();
  }

  let output = doing.run(["view", "done"]).output().expect("failed to run view");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // The "done" view targets the Done section, which has no entries
  assert_eq!(
    helpers::count_entries(&stdout),
    0,
    "view 'done' should show entries from Done section only"
  );
}

#[test]
fn it_respects_view_section_setting() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Current task"]).assert().success();
  doing.run(["done", "Finished task"]).assert().success();

  let output = doing.run(["view", "test"]).output().expect("failed to run view");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Current task"),
    "view scoped to Currently should show current entries"
  );
}

#[test]
#[ignore = "view fuzzy matching not implemented (see #15)"]
fn it_fuzzy_matches_view_names() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing.run(["view", "tes"]).output().expect("failed to run view");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "fuzzy match 'tes' should resolve to 'test' view"
  );
}

#[test]
fn it_returns_error_for_invalid_view_name() {
  let doing = DoingCmd::new_with_config(VIEW_CONFIG);

  doing.run(["view", "nonexistent"]).assert().failure();
}
