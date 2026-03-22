use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "our doing does not support --to flag for archive (see #188)"]
fn it_moves_entries_to_specified_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task to move"]).assert().success();
  doing.run(["archive", "--to", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  let later_pos = contents.find("Later:");
  let entry_pos = contents.find("Task to move");

  assert!(
    later_pos.is_some() && entry_pos.is_some() && entry_pos.unwrap() > later_pos.unwrap(),
    "expected entry to be in Later section, got: {contents}"
  );
}

#[test]
#[ignore = "our doing does not support --to flag for archive (see #188)"]
fn it_says_moved_instead_of_archived_for_non_archive_target() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task to move"]).assert().success();

  let output = doing.run(["archive", "--to", "Later"]).output().expect("failed to run");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("Moved"),
    "expected 'Moved' message for non-Archive target, got: {stderr}"
  );
}

#[test]
#[ignore = "our doing does not support --to flag for archive (see #188)"]
fn it_says_archived_when_target_is_archive() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task to archive"]).assert().success();

  let output = doing
    .run(["archive", "--to", "Archive"])
    .output()
    .expect("failed to run");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("Archived"),
    "expected 'Archived' message when target is Archive, got: {stderr}"
  );
}

#[test]
#[ignore = "our doing does not support --to flag for archive (see #188)"]
fn it_adds_from_label_regardless_of_target() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task to move"]).assert().success();
  doing.run(["archive", "--to", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@from(Currently)"),
    "expected @from(Currently) label on moved entry, got: {contents}"
  );
}
