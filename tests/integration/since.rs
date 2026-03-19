use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_filters_by_time_within_a_day() {
  let doing = DoingCmd::new();

  doing.run(["done", "Today entry"]).assert().success();
  doing
    .run(["done", "--back", "yesterday 4pm", "Yesterday 4pm entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "yesterday 3pm", "Yesterday 3pm entry"])
    .assert()
    .success();

  let output = doing
    .run(["since", "yesterday 3:30pm"])
    .output()
    .expect("failed to run since");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "since yesterday 3:30pm should display 2 entries"
  );
  assert!(
    !stdout.contains("Yesterday 3pm entry"),
    "entry from before 3:30pm should not be shown"
  );
}

#[test]
fn it_shows_all_entries_in_wide_range() {
  let doing = DoingCmd::new();

  doing.run(["done", "Today entry"]).assert().success();
  doing
    .run(["done", "--back", "yesterday 3pm", "Yesterday entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "2 days ago", "Two days ago entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "3 days ago 1pm", "Three days ago entry"])
    .assert()
    .success();

  let output = doing
    .run(["since", "4 days ago"])
    .output()
    .expect("failed to run since");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    4,
    "since 4 days ago should display all 4 entries"
  );
}

#[test]
fn it_shows_entries_since_yesterday() {
  let doing = DoingCmd::new();

  doing.run(["done", "Today entry"]).assert().success();
  doing
    .run(["done", "--back", "yesterday 3pm", "Yesterday 3pm entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "yesterday 4pm", "Yesterday 4pm entry"])
    .assert()
    .success();
  doing
    .run(["done", "--back", "2 days ago", "Two days ago entry"])
    .assert()
    .success();

  let output = doing.run(["since", "yesterday"]).output().expect("failed to run since");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    3,
    "since yesterday should display 3 entries"
  );
  assert!(
    !stdout.contains("Two days ago entry"),
    "entry from 2 days ago should not be shown"
  );
}
