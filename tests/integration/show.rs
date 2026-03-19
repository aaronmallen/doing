use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_displays_entries_from_default_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one @tag1"]).assert().success();
  doing.run(["now", "Entry two @tag2"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show should display 2 entries from default section"
  );
  assert!(stdout.contains("Entry one"), "output should contain first entry");
  assert!(stdout.contains("Entry two"), "output should contain second entry");
}

#[test]
fn it_excludes_tagged_entries_with_bool_not() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry with tag1 @tag1"]).assert().success();
  doing.run(["now", "Entry with tag2 @tag2"]).assert().success();
  doing
    .run(["now", "Entry with both @tag1 @tag2 @tag3"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--tag", "tag2", "--bool", "not"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show --tag tag2 --bool not should display 1 entry"
  );
  assert!(
    stdout.contains("Entry with tag1"),
    "output should contain the entry without tag2"
  );
}

#[test]
fn it_filters_entries_by_multiple_tags_with_bool_and() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry with tag1 @tag1"]).assert().success();
  doing.run(["now", "Entry with tag2 @tag2"]).assert().success();
  doing.run(["now", "Entry with both @tag1 @tag2"]).assert().success();

  let output = doing
    .run(["show", "--tag", "tag1", "--tag", "tag2", "--bool", "and"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show --tag tag1 --tag tag2 --bool and should display 1 entry"
  );
  assert!(
    stdout.contains("Entry with both"),
    "output should contain the entry with both tags"
  );
}

#[test]
fn it_filters_entries_by_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Tagged entry @tag1"]).assert().success();
  doing.run(["now", "Untagged entry"]).assert().success();

  let output = doing
    .run(["show", "--tag", "tag1"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show --tag tag1 should display 1 entry"
  );
  assert!(
    stdout.contains("Tagged entry"),
    "output should contain the tagged entry"
  );
}

#[test]
fn it_limits_entries_with_count_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();
  doing.run(["now", "Entry three"]).assert().success();

  let output = doing
    .run(["show", "--count", "2"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show --count 2 should display exactly 2 entries"
  );
}

#[test]
fn it_shows_entries_from_all_sections() {
  let doing = DoingCmd::new();

  doing.run(["now", "Default section entry"]).assert().success();
  doing
    .run(["now", "--section", "Other", "Other section entry"])
    .assert()
    .success();

  let output = doing.run(["show", "all"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show all should display entries from all sections"
  );
  assert!(
    stdout.contains("Default section entry"),
    "output should contain default section entry"
  );
  assert!(
    stdout.contains("Other section entry"),
    "output should contain other section entry"
  );
}

#[test]
fn it_sorts_entries_ascending() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "2h ago", "Older entry"]).assert().success();
  doing.run(["now", "Newer entry"]).assert().success();

  let output = doing
    .run(["show", "--sort", "asc"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().filter(|l| l.contains('|')).collect();

  assert!(lines.len() >= 2, "should have at least 2 entry lines");
  assert!(
    lines[0].contains("Older entry"),
    "first entry in asc sort should be older"
  );
  assert!(
    lines[1].contains("Newer entry"),
    "second entry in asc sort should be newer"
  );
}

#[test]
fn it_sorts_entries_descending() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "2h ago", "Older entry"]).assert().success();
  doing.run(["now", "Newer entry"]).assert().success();

  let output = doing
    .run(["show", "--sort", "desc"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().filter(|l| l.contains('|')).collect();

  assert!(lines.len() >= 2, "should have at least 2 entry lines");
  assert!(
    lines[0].contains("Newer entry"),
    "first entry in desc sort should be newer"
  );
  assert!(
    lines[1].contains("Older entry"),
    "second entry in desc sort should be older"
  );
}
