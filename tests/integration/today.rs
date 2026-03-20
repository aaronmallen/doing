use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_excludes_entries_from_yesterday() {
  let doing = DoingCmd::new();

  doing.run(["done", "Today entry @tag1"]).assert().success();
  doing.run(["now", "Another today entry @tag2"]).assert().success();
  doing
    .run(["done", "--back", "24h", "Yesterday should not show up"])
    .assert()
    .success();

  let output = doing.run(["today"]).output().expect("failed to run today");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "today should display 2 entries");
  assert!(
    !stdout.contains("Yesterday should not show up"),
    "entry from yesterday should not be shown"
  );
}

#[test]
fn it_shows_entries_from_today() {
  let doing = DoingCmd::new();

  doing.run(["done", "Test entry one @tag1"]).assert().success();
  doing.run(["now", "Test entry two @tag2"]).assert().success();

  let output = doing.run(["today"]).output().expect("failed to run today");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 2, "today should display 2 entries");
  assert!(stdout.contains("Test entry one"), "output should contain first entry");
  assert!(stdout.contains("Test entry two"), "output should contain second entry");
}
