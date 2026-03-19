use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_limits_entries_with_count_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();
  doing.run(["now", "Entry three"]).assert().success();

  let output = doing
    .run(["recent", "--count", "2"])
    .output()
    .expect("failed to run recent");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "recent --count 2 should display exactly 2 entries"
  );
}

#[test]
fn it_scopes_to_specific_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Default entry"]).assert().success();
  doing
    .run(["now", "--section", "Other", "Other entry"])
    .assert()
    .success();

  let output = doing
    .run(["recent", "--section", "Other"])
    .output()
    .expect("failed to run recent");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "recent --section Other should display 1 entry"
  );
  assert!(
    stdout.contains("Other entry"),
    "output should contain the Other section entry"
  );
  assert!(
    !stdout.contains("Default entry"),
    "output should not contain the default section entry"
  );
}

#[test]
fn it_shows_recent_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry one @tag1"]).assert().success();
  doing.run(["now", "Test entry two @tag2"]).assert().success();

  let output = doing.run(["recent"]).output().expect("failed to run recent");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "recent should display 2 entries");
  assert!(stdout.contains("Test entry one"), "output should contain first entry");
  assert!(stdout.contains("Test entry two"), "output should contain second entry");
}

#[test]
fn it_uses_default_count_from_config() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[templates.recent]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
count = 2

[editors]
default = "cat"
"#;
  let doing = DoingCmd::new_with_config(config);

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();
  doing.run(["now", "Entry three"]).assert().success();

  let output = doing.run(["recent"]).output().expect("failed to run recent");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "recent with config count=2 should display exactly 2 entries"
  );
}
