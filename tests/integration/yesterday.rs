use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_excludes_entries_from_today() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--took", "30m", "--back", "yesterday 3pm", "Yesterday entry"])
    .assert()
    .success();
  doing.run(["now", "Today should not show up"]).assert().success();

  let output = doing.run(["yesterday"]).output().expect("failed to run yesterday");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 1, "yesterday should display 1 entry");
  assert!(
    !stdout.contains("Today should not show up"),
    "entry from today should not be shown"
  );
}

#[test]
#[ignore] // requires shorthand duration support (https://github.com/aaronmallen/doing/issues/14)
fn it_excludes_entries_from_two_days_ago() {
  let doing = DoingCmd::new();

  doing
    .run(["done", "--took", "30m", "--back", "yesterday 3pm", "Yesterday entry"])
    .assert()
    .success();
  doing.run(["now", "Today entry"]).assert().success();
  doing
    .run(["done", "--back", "48h", "Two days ago should not show up"])
    .assert()
    .success();

  let output = doing.run(["yesterday"]).output().expect("failed to run yesterday");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 1, "yesterday should display 1 entry");
  assert!(
    !stdout.contains("Two days ago should not show up"),
    "entry from 2 days ago should not be shown"
  );
  assert!(!stdout.contains("Today entry"), "entry from today should not be shown");
}

#[test]
fn it_shows_entries_from_yesterday() {
  let doing = DoingCmd::new();

  doing
    .run([
      "done",
      "--took",
      "30m",
      "--back",
      "yesterday 3pm",
      "Adding an entry finished yesterday",
    ])
    .assert()
    .success();

  let output = doing.run(["yesterday"]).output().expect("failed to run yesterday");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 1, "yesterday should display 1 entry");
  assert!(
    stdout.contains("Adding an entry finished yesterday"),
    "output should contain yesterday's entry"
  );
}
