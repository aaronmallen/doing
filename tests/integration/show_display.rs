use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_ends_output_with_trailing_newline() {
  let doing = DoingCmd::new();

  doing.run(["now", "Trailing newline test"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.ends_with('\n'),
    "display output should end with a trailing newline"
  );
}

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
fn it_shows_tags_in_entry_display() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Working on feature @coding @project(myapp)"])
    .assert()
    .success();

  // Verify tags appear in show output
  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("@coding"), "expected @coding in show output");
  assert!(
    stdout.contains("@project(myapp)"),
    "expected @project(myapp) in show output"
  );

  // Verify tags appear in last output
  let output = doing.run(["last"]).output().expect("failed to run last");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("@coding"), "expected @coding in last output");
  assert!(
    stdout.contains("@project(myapp)"),
    "expected @project(myapp) in last output"
  );
}

#[test]
fn it_shows_tags_in_info_messages() {
  let doing = DoingCmd::new();

  // Create entry with tags and check info message (on stderr)
  let output = doing
    .run(["now", "Working on feature @coding"])
    .output()
    .expect("failed to run now");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(stderr.contains("@coding"), "expected @coding in now info message");

  // Finish the entry and check info message (on stderr)
  let output = doing.run(["done"]).output().expect("failed to run done");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(stderr.contains("@coding"), "expected @coding in done info message");
}

#[test]
fn it_shows_time_intervals_by_default() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "2h ago", "Timed task"]).assert().success();
  doing.run(["finish", "--took", "1h"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_line = stdout
    .lines()
    .find(|l| l.contains("Timed task"))
    .expect("should contain the entry");

  assert!(
    entry_line.contains(':'),
    "entry should include a time interval by default, got: {entry_line}"
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
