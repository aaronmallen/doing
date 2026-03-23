use std::fs;

use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp,
};

#[test]
fn it_sets_start_and_done_from_range() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected_start = format!("{today} 13:00");
  let expected_done = format!("{today} 15:00");

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task from range\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--from", "1pm to 3pm"]).assert().success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be 1pm");
  assert_times_within_tolerance(&done_time, &expected_done, 1, "@done should be 3pm");
}
