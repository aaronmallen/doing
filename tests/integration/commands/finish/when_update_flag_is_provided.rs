use std::fs;

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_done_timestamp, fmt_time};

#[test]
fn it_overwrites_existing_done_date() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task to update @done(2026-03-20 15:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--update"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@done(2026-03-20 15:00)"),
    "expected old @done date to be replaced, got: {contents}"
  );

  let done_time = extract_done_timestamp(&contents);
  assert_times_within_tolerance(&done_time, &now, 1, "@done should be updated to now");
}
