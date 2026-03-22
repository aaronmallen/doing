use std::fs;

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_done_timestamp};

#[test]
fn it_sets_done_date_to_specified_time() {
  let doing = DoingCmd::new();
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();
  let expected_done = format!("{today} 11:30");

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task to finish at time\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--at", "11:30am"]).assert().success();

  let contents = doing.read_doing_file();
  let done_time = extract_done_timestamp(&contents);

  assert_times_within_tolerance(&done_time, &expected_done, 1, "@done should be 11:30am");
}
