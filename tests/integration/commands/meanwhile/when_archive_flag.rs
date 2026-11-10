use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_archives_finished_meanwhile() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Old MW @meanwhile\n\nArchive:\n",
  )
  .expect("failed to write doing file");

  doing.run(["meanwhile", "--archive", "New MW"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should be in Currently with @meanwhile
  let currently_pos = contents.find("Currently:").expect("expected Currently section");
  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let new_entry_pos = contents.find("New MW").expect("expected New MW entry");
  assert!(
    new_entry_pos > currently_pos && new_entry_pos < archive_pos,
    "expected New MW in Currently section, got: {contents}"
  );

  let new_line = contents
    .lines()
    .find(|l| l.contains("New MW"))
    .expect("expected New MW line");
  assert!(
    new_line.contains("@meanwhile"),
    "expected @meanwhile on new entry, got: {new_line}"
  );

  // Old entry should be in Archive with @done
  let old_entry_pos = contents.find("Old MW").expect("expected Old MW entry");
  assert!(
    old_entry_pos > archive_pos,
    "expected Old MW in Archive section, got: {contents}"
  );

  let old_line = contents
    .lines()
    .find(|l| l.contains("Old MW"))
    .expect("expected Old MW line");
  assert!(
    old_line.contains("@done("),
    "expected @done on archived entry, got: {old_line}"
  );
}

#[test]
fn it_archives_finished_meanwhile_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 10:00 | Old MW @meanwhile\n\nArchive:\n",
  )
  .expect("failed to write doing file");

  doing.run(["meanwhile", "-a", "New MW short"]).assert().success();

  let contents = doing.read_doing_file();

  // New entry should be in Currently with @meanwhile
  let archive_pos = contents.find("Archive:").expect("expected Archive section");
  let new_entry_pos = contents.find("New MW short").expect("expected New MW short entry");
  assert!(
    new_entry_pos < archive_pos,
    "expected new entry in Currently section, got: {contents}"
  );

  // Old entry should be in Archive with @done
  let old_entry_pos = contents.find("Old MW").expect("expected Old MW entry");
  assert!(
    old_entry_pos > archive_pos,
    "expected Old MW in Archive section, got: {contents}"
  );
}
