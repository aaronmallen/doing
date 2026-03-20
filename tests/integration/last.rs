use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_filters_by_search_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "jumping jesus @tag1"]).assert().success();
  doing.run(["now", "sad monkey @tag2"]).assert().success();
  doing.run(["now", "burly man @tag3"]).assert().success();

  let output = doing
    .run(["last", "--search", "jumping jesus"])
    .output()
    .expect("failed to run last");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "last --search should display exactly one entry"
  );
  assert!(
    stdout.contains("jumping jesus"),
    "returned entry should contain the search keyword"
  );
}

#[test]
fn it_filters_by_tag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Entry with unique tag @balloonpants"])
    .assert()
    .success();
  doing.run(["now", "Entry with other tag @tag2"]).assert().success();
  doing.run(["now", "Entry with third tag @tag3"]).assert().success();

  let output = doing
    .run(["last", "--tag", "balloonpants"])
    .output()
    .expect("failed to run last");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "last --tag should display exactly one entry"
  );
  assert!(
    stdout.contains("Entry with unique tag"),
    "returned entry should be the one matching the filtered tag"
  );
}

#[test]
fn it_shows_the_most_recent_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();
  doing.run(["now", "Most recent entry @tag1"]).assert().success();

  let output = doing.run(["last"]).output().expect("failed to run last");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "last should display exactly one entry"
  );
  assert!(
    stdout.contains("Most recent entry"),
    "last should show the most recently added entry"
  );
}

#[test]
fn it_skips_done_entries_when_selecting_last() {
  let doing = DoingCmd::new();

  // Create an active entry
  doing.run(["now", "Active task"]).assert().success();

  // Create and immediately finish another entry
  doing.run(["now", "Finished task"]).assert().success();
  doing.run(["done"]).assert().success();

  // `last` should show the active (unfinished) entry
  let output = doing.run(["last"]).output().expect("failed to run last");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Active task"), "expected last to show unfinished entry");
  assert!(!stdout.contains("Finished task"), "expected last to skip @done entry");
}
