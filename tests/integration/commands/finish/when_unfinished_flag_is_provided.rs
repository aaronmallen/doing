use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_only_finishes_entries_without_done() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Already done @done(2026-03-22 10:00)\n\t- 2026-03-22 09:00 | Not done yet\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();

  // Already done entry should keep its original @done date
  let done_line = contents.lines().find(|l| l.contains("Already done")).unwrap();
  assert!(
    done_line.contains("@done(2026-03-22 10:00)"),
    "expected already-done entry to keep original date, got: {done_line}"
  );

  // Unfinished entry should now be done
  let undone_line = contents.lines().find(|l| l.contains("Not done yet")).unwrap();
  assert!(
    undone_line.contains("@done("),
    "expected unfinished entry to be marked done, got: {undone_line}"
  );
}
