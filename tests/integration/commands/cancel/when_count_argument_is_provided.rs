use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_cancels_n_most_recent_entries() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Task A\n\t- 2026-03-22 14:00 | Task B\n\t- 2026-03-22 13:00 | Task C\n",
  )
  .expect("failed to write doing file");

  doing.run(["cancel", "2"]).assert().success();

  let contents = doing.read_doing_file();
  let done_count = contents.lines().filter(|l| l.contains("@done")).count();
  assert_eq!(
    done_count, 2,
    "expected 2 cancelled entries, got {done_count} in: {contents}"
  );

  // None should have timestamps
  for line in contents.lines().filter(|l| l.contains("@done")) {
    assert!(
      !line.contains("@done("),
      "expected @done without timestamp, got: {line}"
    );
  }
}
