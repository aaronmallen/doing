use std::fs;

use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp, fmt_time,
};

#[test]
fn it_adjusts_start_time_for_elapsed_duration() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());
  let expected_start = fmt_time(chrono::Local::now() - chrono::Duration::minutes(45));

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task took 45m\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--took", "45m"]).assert().success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be now - 45m");
  assert_times_within_tolerance(&done_time, &now, 1, "@done should be now");
}
