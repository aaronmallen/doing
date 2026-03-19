use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_shows_entries_from_today() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test new entry @tag1"]).assert().success();
  doing.run(["now", "Test new entry 2 @tag2"]).assert().success();

  let output = doing.run(["on", "today"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "on today should display 2 entries");
}

#[test]
fn it_shows_entries_from_yesterday() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "yesterday 3pm", "Yesterday entry"])
    .assert()
    .success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["on", "yesterday"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "on yesterday should display 1 entry"
  );
  assert!(
    stdout.contains("Yesterday entry"),
    "output should contain yesterday's entry"
  );
  assert!(!stdout.contains("Today entry"), "entry from today should not be shown");
}

#[test]
fn it_works_with_natural_language_dates() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--back", "last monday 10am", "Monday entry"])
    .assert()
    .success();
  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["on", "last monday"]).output().expect("failed to run on");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Monday entry"),
    "on last monday should include the monday entry"
  );
  assert!(!stdout.contains("Today entry"), "entry from today should not be shown");
}
