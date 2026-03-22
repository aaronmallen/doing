use std::fs;

use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp, fmt_time,
};

#[test]
fn it_backdates_start_time() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());
  let expected_start = fmt_time(chrono::Local::now() - chrono::Duration::hours(1));

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task back 1h\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--back", "1 hour ago"]).assert().success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be ~1 hour ago");
  assert_times_within_tolerance(&done_time, &now, 1, "@done should be now");
}
