use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_filters_entries_with_only_timed() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "2h ago", "Finished task @coding"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();
  doing.run(["now", "Unfinished task @writing"]).assert().success();

  let output = doing
    .run(["show", "--only-timed"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show --only_timed should display only entries with a time interval"
  );
  assert!(stdout.contains("Finished task"), "should include the finished entry");
  assert!(
    !stdout.contains("Unfinished task"),
    "should exclude the unfinished entry"
  );
}

#[test]
fn it_filters_entries_with_val_date_query() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "5 days ago", "Old task @work"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "3 days ago", "Middle task @work"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1 day ago", "Recent task @work"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--val", "date > 4 days ago", "--val", "date < 2 days ago"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show --val with date range should match only the middle entry"
  );
  assert!(stdout.contains("Middle task"), "should include the entry within range");
  assert!(!stdout.contains("Old task"), "should exclude the old entry");
  assert!(!stdout.contains("Recent task"), "should exclude the recent entry");
}

#[test]
fn it_filters_entries_with_val_tag_query() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Task with progress @progress(75)"])
    .assert()
    .success();
  doing
    .run(["now", "Task with low progress @progress(25)"])
    .assert()
    .success();
  doing.run(["now", "Task without progress @other"]).assert().success();

  let output = doing
    .run(["show", "--val", "progress >= 50"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show --val 'progress >= 50' should match 1 entry"
  );
  assert!(
    stdout.contains("Task with progress"),
    "should include entry with progress >= 50"
  );
}

#[test]
fn it_shows_tag_totals_with_totals_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h ago", "Coding task @coding"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();
  doing
    .run(["now", "--back", "2h ago", "Writing task @writing"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing.run(["show", "--totals"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("Tag Totals"), "output should contain tag totals header");
  assert!(stdout.contains("coding:"), "totals should include coding tag");
  assert!(stdout.contains("writing:"), "totals should include writing tag");
  assert!(
    stdout.contains("Total tracked:"),
    "totals should include total tracked line"
  );
}

#[test]
#[ignore = "show command --times flag is not wired into rendering (see plan to implement --times display)"]
fn it_shows_time_intervals_with_times_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "2h ago", "Timed task"]).assert().success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing.run(["show", "--times"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_line = stdout
    .lines()
    .find(|l| l.contains("Timed task"))
    .expect("should contain the entry");

  assert!(
    entry_line.contains("1:00:00") || entry_line.contains("1h") || entry_line.contains("01:00"),
    "entry should include a time interval when --times is used"
  );
}

#[test]
fn it_shows_totals_only_for_timed_entries() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "2h ago", "Finished @coding"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();
  doing.run(["now", "Unfinished @writing"]).assert().success();

  let output = doing.run(["show", "--totals"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("Tag Totals"), "output should contain tag totals");
  assert!(
    stdout.contains("coding:"),
    "totals should include coding tag from finished entry"
  );
}

#[test]
#[ignore = "show command missing --tag_sort CLI flag (see plan to add --tag_sort/--tag_order flags)"]
fn it_sorts_totals_by_tag_name_ascending() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h ago", "Zebra task @zebra"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();
  doing
    .run(["now", "--back", "2h ago", "Alpha task @alpha"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing
    .run(["show", "--totals", "--tag_sort", "name", "--tag_order", "asc"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let totals_section = stdout.split("--- Tag Totals ---").nth(1).expect("should have totals");
  let first_tag = totals_section
    .lines()
    .find(|l| l.contains(':'))
    .expect("should have tag lines");

  assert!(first_tag.starts_with("alpha"), "first tag in name asc should be alpha");
}

#[test]
#[ignore = "show command missing --tag_sort CLI flag (see plan to add --tag_sort/--tag_order flags)"]
fn it_sorts_totals_by_tag_name_descending() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h ago", "Zebra task @zebra"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();
  doing
    .run(["now", "--back", "2h ago", "Alpha task @alpha"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing
    .run(["show", "--totals", "--tag_sort", "name", "--tag_order", "desc"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let totals_section = stdout.split("--- Tag Totals ---").nth(1).expect("should have totals");
  let first_tag = totals_section
    .lines()
    .find(|l| l.contains(':'))
    .expect("should have tag lines");

  assert!(first_tag.starts_with("zebra"), "first tag in name desc should be zebra");
}

#[test]
#[ignore = "show command missing --tag_sort CLI flag (see plan to add --tag_sort/--tag_order flags)"]
fn it_sorts_totals_by_time_descending() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "4h ago", "Long task @coding"])
    .assert()
    .success();
  doing.run(["finish", "--took", "3h"]).assert().success();
  doing
    .run(["now", "--back", "2h ago", "Short task @writing"])
    .assert()
    .success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing
    .run(["show", "--totals", "--tag_sort", "time", "--tag_order", "desc"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let totals_section = stdout.split("--- Tag Totals ---").nth(1).expect("should have totals");
  let first_tag = totals_section
    .lines()
    .find(|l| l.contains(':'))
    .expect("should have tag lines");

  assert!(
    first_tag.starts_with("coding"),
    "first tag sorted by time desc should be coding (longest)"
  );
}
