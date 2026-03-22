use std::fs;

use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, fmt_time};

#[test]
#[ignore = "finish --auto not implemented (see #168)"]
fn it_generates_done_dates_from_next_entry_start() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Task C\n\t- 2026-03-22 10:00 | Task B\n\t- 2026-03-22 09:00 | Task A\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--auto", "3"]).assert().success();

  let contents = doing.read_doing_file();

  // Task A @done should be 1 min before Task B start (09:59)
  let task_a_line = contents.lines().find(|l| l.contains("Task A")).unwrap();
  assert!(
    task_a_line.contains("@done(2026-03-22 09:59)"),
    "expected Task A @done = 09:59 (1 min before Task B), got: {task_a_line}"
  );

  // Task B @done should be 1 min before Task C start (10:59)
  let task_b_line = contents.lines().find(|l| l.contains("Task B")).unwrap();
  assert!(
    task_b_line.contains("@done(2026-03-22 10:59)"),
    "expected Task B @done = 10:59 (1 min before Task C), got: {task_b_line}"
  );

  // Task C (last) @done should be now
  let re = regex::Regex::new(r"Task C.*@done\((\d{4}-\d{2}-\d{2} \d{2}:\d{2})\)").unwrap();
  let cap = re.captures(&contents).expect("expected Task C to have @done");
  let task_c_done = &cap[1];
  assert_times_within_tolerance(task_c_done, &now, 1, "Task C @done should be now");
}

#[test]
#[ignore = "finish --auto not implemented (see #168)"]
fn it_overrides_date_and_back_parameters() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Task B\n\t- 2026-03-22 09:00 | Task A\n",
  )
  .expect("failed to write doing file");

  // --auto should take precedence over --back
  doing.run(["finish", "--auto", "--back", "2pm", "2"]).assert().success();

  let contents = doing.read_doing_file();

  // Task A @done should still be 1 min before Task B (09:59), not affected by --back
  let task_a_line = contents.lines().find(|l| l.contains("Task A")).unwrap();
  assert!(
    task_a_line.contains("@done(2026-03-22 09:59)"),
    "expected --auto to override --back for Task A, got: {task_a_line}"
  );
}
