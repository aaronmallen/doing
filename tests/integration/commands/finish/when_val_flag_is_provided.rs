use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_finishes_entries_matching_tag_value() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Task high @progress(80)\n\t- 2026-03-22 10:00 | Task low @progress(30)\n\t- 2026-03-22 09:00 | Task mid @progress(65)\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--val", "progress > 60", "5"]).assert().success();

  let contents = doing.read_doing_file();

  let task_high = contents.lines().find(|l| l.contains("Task high")).unwrap();
  assert!(
    task_high.contains("@done("),
    "expected Task high (@progress(80)) to be finished, got: {task_high}"
  );

  let task_low = contents.lines().find(|l| l.contains("Task low")).unwrap();
  assert!(
    !task_low.contains("@done"),
    "expected Task low (@progress(30)) to remain unfinished, got: {task_low}"
  );

  let task_mid = contents.lines().find(|l| l.contains("Task mid")).unwrap();
  assert!(
    task_mid.contains("@done("),
    "expected Task mid (@progress(65)) to be finished, got: {task_mid}"
  );
}
