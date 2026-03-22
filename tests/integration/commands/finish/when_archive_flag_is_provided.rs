use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "finish --archive does not add @from(Section) tag (see #169)"]
fn it_finishes_and_archives_entry() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 09:00 | Task to archive\n\nArchive:\n",
  )
  .expect("failed to write doing file");

  doing.run(["finish", "--archive"]).assert().success();

  let contents = doing.read_doing_file();

  // Entry should be moved to Archive section
  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let entry_pos = contents.find("Task to archive").expect("expected entry");
  assert!(
    entry_pos > archive_pos,
    "expected entry to be in Archive section, got: {contents}"
  );

  // Entry should have @done
  let entry_line = contents
    .lines()
    .find(|l| l.contains("Task to archive"))
    .expect("expected entry line");
  assert!(
    entry_line.contains("@done("),
    "expected @done on archived entry, got: {entry_line}"
  );

  // Entry should have @from(Currently)
  assert!(
    entry_line.contains("@from(Currently)"),
    "expected @from(Currently) on archived entry, got: {entry_line}"
  );
}
