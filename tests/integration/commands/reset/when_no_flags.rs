use std::fs;

use crate::support::helpers::{DoingCmd, extract_entry_timestamp, fmt_time};

#[test]
fn it_is_accessible_via_begin_alias() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 10:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["begin"]).assert().success();

  let contents = doing.read_doing_file();
  let timestamp = extract_entry_timestamp(&contents);
  let now = fmt_time(chrono::Local::now());

  // Start time should be close to current time
  crate::support::helpers::assert_times_within_tolerance(&timestamp, &now, 2, "begin alias reset time");
}

#[test]
fn it_removes_done_tag_by_default() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Task A @done(2026-03-22 11:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@done"),
    "expected @done to be removed by default, got: {contents}"
  );
}

#[test]
fn it_resets_start_time_of_last_entry() {
  let doing = DoingCmd::new();

  fs::write(doing.doing_file_path(), "Currently:\n\t- 2026-03-22 10:00 | Task A\n")
    .expect("failed to write doing file");

  doing.run(["reset"]).assert().success();

  let contents = doing.read_doing_file();
  let timestamp = extract_entry_timestamp(&contents);
  let now = fmt_time(chrono::Local::now());

  crate::support::helpers::assert_times_within_tolerance(&timestamp, &now, 2, "reset start time");
}
