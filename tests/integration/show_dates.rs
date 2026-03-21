use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_combines_date_and_tag_filters() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "5 days ago", "Old tagged @project"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 day ago", "Recent tagged @project"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 day ago", "Recent untagged"])
    .assert()
    .success();
  doing.run(["now", "Current tagged @project"]).assert().success();

  let output = doing
    .run(["show", "--after", "2 days ago", "--tag", "project"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show --after + --tag should apply both filters"
  );
  assert!(stdout.contains("Recent tagged"), "should include recent tagged entry");
  assert!(stdout.contains("Current tagged"), "should include current tagged entry");
  assert!(!stdout.contains("Old tagged"), "should exclude old tagged entry");
  assert!(!stdout.contains("Recent untagged"), "should exclude untagged entry");
}

#[test]
fn it_filters_entries_by_after_date() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "5 days ago", "Old entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 day ago", "Recent entry"])
    .assert()
    .success();
  doing.run(["now", "Current entry"]).assert().success();

  let output = doing
    .run(["show", "--after", "2 days ago"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show --after should exclude entries before the cutoff"
  );
  assert!(stdout.contains("Recent entry"), "should include recent entry");
  assert!(stdout.contains("Current entry"), "should include current entry");
  assert!(!stdout.contains("Old entry"), "should exclude old entry");
}

#[test]
fn it_filters_entries_by_age_newest() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "3h ago", "Entry one"]).assert().success();
  doing.run(["now", "--back", "2h ago", "Entry two"]).assert().success();
  doing.run(["now", "--back", "1h ago", "Entry three"]).assert().success();
  doing.run(["now", "Entry four"]).assert().success();

  let output = doing
    .run(["show", "--count", "2", "--age", "newest"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "should show exactly 2 entries");
  assert!(stdout.contains("Entry three"), "should include third entry");
  assert!(stdout.contains("Entry four"), "should include fourth entry");
  assert!(!stdout.contains("Entry one"), "should exclude first entry");
  assert!(!stdout.contains("Entry two"), "should exclude second entry");
}

#[test]
fn it_filters_entries_by_age_oldest() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "3h ago", "Entry one"]).assert().success();
  doing.run(["now", "--back", "2h ago", "Entry two"]).assert().success();
  doing.run(["now", "--back", "1h ago", "Entry three"]).assert().success();
  doing.run(["now", "Entry four"]).assert().success();

  let output = doing
    .run(["show", "--count", "2", "--age", "oldest"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "should show exactly 2 entries");
  assert!(stdout.contains("Entry one"), "should include first entry");
  assert!(stdout.contains("Entry two"), "should include second entry");
  assert!(!stdout.contains("Entry three"), "should exclude third entry");
  assert!(!stdout.contains("Entry four"), "should exclude fourth entry");
}

#[test]
fn it_filters_entries_by_before_date() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "5 days ago", "Old entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "3 days ago", "Middle entry"])
    .assert()
    .success();
  doing.run(["now", "Current entry"]).assert().success();

  let output = doing
    .run(["show", "--before", "2 days ago"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show --before should exclude entries after the cutoff"
  );
  assert!(stdout.contains("Old entry"), "should include old entry");
  assert!(stdout.contains("Middle entry"), "should include middle entry");
  assert!(!stdout.contains("Current entry"), "should exclude current entry");
}

#[test]
fn it_filters_entries_by_date_range_with_before_and_after() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "6 days ago", "Too old"]).assert().success();
  doing
    .run(["now", "--back", "4 days ago", "In range one"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "3 days ago", "In range two"])
    .assert()
    .success();
  doing.run(["now", "Too new"]).assert().success();

  let output = doing
    .run(["show", "--after", "5 days ago", "--before", "2 days ago"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show --after + --before should show only entries in range"
  );
  assert!(stdout.contains("In range one"), "should include first in-range entry");
  assert!(stdout.contains("In range two"), "should include second in-range entry");
  assert!(!stdout.contains("Too old"), "should exclude entry before range");
  assert!(!stdout.contains("Too new"), "should exclude entry after range");
}

#[test]
fn it_filters_entries_by_from_date_range() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "6 days ago", "Too old"]).assert().success();
  doing
    .run(["now", "--back", "4 days ago", "In range one"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "3 days ago", "In range two"])
    .assert()
    .success();
  doing.run(["now", "Too new"]).assert().success();

  let output = doing
    .run(["show", "--from", "5 days ago to 2 days ago"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show --from with range should show only entries in range"
  );
  assert!(stdout.contains("In range one"), "should include first in-range entry");
  assert!(stdout.contains("In range two"), "should include second in-range entry");
  assert!(!stdout.contains("Too old"), "should exclude entry before range");
  assert!(!stdout.contains("Too new"), "should exclude entry after range");
}

#[test]
fn it_filters_entries_by_from_single_date() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "3 days ago", "Too old"]).assert().success();
  doing
    .run(["now", "--back", "1 day ago 10am", "Yesterday entry"])
    .assert()
    .success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing
    .run(["show", "--from", "yesterday"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show --from with single date should show only entries from that day"
  );
  assert!(stdout.contains("Yesterday entry"), "should include yesterday's entry");
  assert!(!stdout.contains("Too old"), "should exclude older entries");
  assert!(!stdout.contains("Today entry"), "should exclude today's entry");
}
