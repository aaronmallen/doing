use chrono::{Duration, Local};

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_backdates_new_entry() {
  let doing = DoingCmd::new();
  let expected_time = fmt_time(Local::now() - Duration::minutes(30));

  doing
    .run(["meanwhile", "--back", "30 minutes ago", "Backdated MW"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Backdated MW"),
    "expected backdated entry, got: {contents}"
  );
  assert!(
    contents.contains("@meanwhile"),
    "expected @meanwhile tag, got: {contents}"
  );

  let entry_time = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(&entry_time, &expected_time, 1, "backdated meanwhile start time");
}

#[test]
fn it_backdates_with_short_flag() {
  let doing = DoingCmd::new();
  let expected_time = fmt_time(Local::now() - Duration::hours(1));

  doing
    .run(["meanwhile", "-b", "1 hour ago", "Short back MW"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Short back MW"),
    "expected backdated entry, got: {contents}"
  );

  let entry_time = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(
    &entry_time,
    &expected_time,
    1,
    "short flag backdated meanwhile start time",
  );
}
