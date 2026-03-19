use std::fs;

use pretty_assertions::assert_eq;

use crate::helpers::{DoingCmd, count_entries};

#[test]
fn it_creates_backup_files_in_backup_directory() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();

  let backup_dir = doing.backup_dir();
  assert!(backup_dir.exists(), "backup directory should be created");

  let backups: Vec<_> = fs::read_dir(backup_dir)
    .expect("should read backup dir")
    .filter_map(|e| e.ok())
    .filter(|e| {
      e.path()
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with(".bak"))
    })
    .collect();

  assert!(
    !backups.is_empty(),
    "backup files should exist after modifying the doing file"
  );
}

#[test]
fn it_restores_previous_state_after_undo() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 2);

  doing.run(["undo"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 1, "undo should restore to one entry");
  assert!(stdout.contains("First entry"), "first entry should remain after undo");
  assert!(
    !stdout.contains("Second entry"),
    "second entry should be gone after undo"
  );
}

#[test]
fn it_restores_undone_change_with_redo() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();

  doing.run(["undo"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    !stdout.contains("Second entry"),
    "second entry should be gone after undo"
  );

  doing.run(["redo"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 2, "redo should restore both entries");
  assert!(stdout.contains("Second entry"), "second entry should return after redo");
}

#[test]
#[ignore = "deviation: sequential undo does not walk backwards through backup history (plan 0112)"]
fn it_undoes_multiple_steps() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();
  doing.run(["now", "Entry three"]).assert().success();

  doing.run(["undo"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 2, "first undo should leave two entries");
  assert!(!stdout.contains("Entry three"), "third entry should be gone");

  doing.run(["undo"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert_eq!(count_entries(&stdout), 1, "second undo should leave one entry");
  assert!(stdout.contains("Entry one"), "first entry should remain");
  assert!(!stdout.contains("Entry two"), "second entry should be gone");
}
