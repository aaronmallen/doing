use chrono::Duration;

use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp, fmt_naive,
  most_recent_clock,
};

#[test]
fn it_sets_start_and_done_from_range() {
  let doing = DoingCmd::new();
  let start = most_recent_clock(13, 0);
  let expected_start = fmt_naive(start);
  let expected_done = fmt_naive(start + Duration::hours(2));

  doing
    .run(["done", "--from", "1pm to 3pm", "From range entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be 1pm");
  assert_times_within_tolerance(&done_time, &expected_done, 1, "@done should be 3pm");
}
