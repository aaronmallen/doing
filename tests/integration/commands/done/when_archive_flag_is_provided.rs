use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_places_entry_directly_in_archive() {
  let doing = DoingCmd::new();

  // Pre-create doing file with Archive section
  fs::write(doing.doing_file_path(), "Currently:\n\nArchive:\n").expect("failed to write doing file");

  doing.run(["now", "Current entry"]).assert().success();
  doing.run(["done", "--archive", "Archive entry"]).assert().success();

  let contents = doing.read_doing_file();

  // Archive entry should be in Archive section
  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let archive_entry_pos = contents.find("Archive entry").expect("expected archive entry");
  assert!(
    archive_entry_pos > archive_pos,
    "expected archive entry under Archive section, got: {contents}"
  );

  // Archive entry should have @done
  let archive_line = contents
    .lines()
    .find(|l| l.contains("Archive entry"))
    .expect("expected Archive entry line");
  assert!(
    archive_line.contains("@done("),
    "expected @done on archived entry, got: {archive_line}"
  );

  // Currently section should still have the current entry, not the archived one
  let currently_pos = contents.find("Currently:").expect("expected Currently section");
  let current_entry_pos = contents.find("Current entry").expect("expected current entry");
  assert!(
    current_entry_pos > currently_pos && current_entry_pos < archive_pos,
    "expected current entry under Currently, not Archive, got: {contents}"
  );
}
