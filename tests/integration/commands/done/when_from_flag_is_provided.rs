use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp,
};

#[test]
#[ignore = "--from range parsing not yet supported (see #160)"]
fn it_sets_start_and_done_from_range() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected_start = format!("{today} 13:00");
  let expected_done = format!("{today} 15:00");

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
