use crate::support::helpers::{
  DoingCmd, assert_times_within_tolerance, extract_done_timestamp, extract_entry_timestamp,
};

#[test]
fn it_sets_finish_date_to_specified_time() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected_done = format!("{today} 15:00");

  doing.run(["done", "--at", "3pm", "At 3pm entry"]).assert().success();

  let contents = doing.read_doing_file();
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&done_time, &expected_done, 1, "@done should be 3pm");
}

#[test]
fn it_combines_with_took_to_backdate_start() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected_start = format!("{today} 14:00");
  let expected_done = format!("{today} 15:00");

  doing
    .run(["done", "--at", "3pm", "--took", "1h", "At+took entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&start_time, &expected_start, 1, "start should be 2pm (3pm - 1h)");
  assert_times_within_tolerance(&done_time, &expected_done, 1, "@done should be 3pm");
}
