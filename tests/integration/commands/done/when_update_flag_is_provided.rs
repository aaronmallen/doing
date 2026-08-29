use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_retags_already_done_entry_with_new_timestamp() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-23 10:00 | Already finished @done(2026-03-23 11:00)\n",
  )
  .expect("failed to write doing file");

  doing.run(["done", "--update"]).assert().success();

  let contents = doing.read_doing_file();
  let entry_line = contents
    .lines()
    .find(|l| l.contains("Already finished"))
    .expect("expected entry in doing file");

  assert!(
    entry_line.contains("@done("),
    "expected @done tag on re-tagged entry, got: {entry_line}"
  );
  assert!(
    !entry_line.contains("@done(2026-03-23 11:00)"),
    "expected new @done timestamp (not original 11:00), got: {entry_line}"
  );
}
