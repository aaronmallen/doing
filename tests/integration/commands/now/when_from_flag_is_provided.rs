use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp,
};

#[test]
#[ignore = "--from single time parsing not yet implemented (see #160)"]
fn it_sets_done_to_end_of_day_when_single_time_given() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", "1pm", "From single time"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &format!("{today} 13:00"), 1, "start should be 1pm");
  assert_times_within_tolerance(&done_time, &format!("{today} 23:59"), 1, "done should be 11:59pm");
}

#[test]
#[ignore = "--from range parsing not yet implemented (see #160)"]
fn it_sets_start_and_done_from_range() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", "1pm to 3pm", "From range entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &format!("{today} 13:00"), 1, "start should be 1pm");
  assert_times_within_tolerance(&done_time, &format!("{today} 15:00"), 1, "done should be 3pm");
}
