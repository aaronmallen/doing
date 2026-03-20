use std::fs;

use assert_cmd::Command;
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
fn it_isolates_undo_history_by_file_path() {
  let doing = DoingCmd::new();
  let temp = doing.temp_dir_path();
  let file_a = temp.join("file_a.md");
  let file_b = temp.join("file_b.md");
  let config_path = temp.join("config.toml");

  let run = |file: &std::path::Path, args: &[&str]| {
    let mut cmd = Command::cargo_bin("doing").unwrap();
    cmd.env("DOING_CONFIG", &config_path);
    cmd.env("DOING_BACKUP_DIR", doing.backup_dir());
    cmd.args(["-f", file.to_str().unwrap(), "--no-color"]);
    cmd.args(args);
    cmd
  };

  // Create entries in file A
  run(&file_a, &["now", "Entry A1"]).assert().success();
  run(&file_a, &["now", "Entry A2"]).assert().success();

  // Create entry in file B
  run(&file_b, &["now", "Entry B1"]).assert().success();

  // Undo last action in file A
  run(&file_a, &["undo"]).assert().success();

  // File A should not contain Entry A2
  let content_a = fs::read_to_string(&file_a).expect("failed to read file A");
  assert!(!content_a.contains("Entry A2"), "undo should have removed Entry A2");
  assert!(content_a.contains("Entry A1"), "Entry A1 should remain");

  // File B should be unaffected
  let content_b = fs::read_to_string(&file_b).expect("failed to read file B");
  assert!(
    content_b.contains("Entry B1"),
    "file B should be unaffected by file A undo"
  );

  // Undo in file B should work independently
  run(&file_b, &["undo"]).assert().success();
  let content_b = fs::read_to_string(&file_b).expect("failed to read file B after undo");
  assert!(
    !content_b.contains("Entry B1"),
    "undo should have removed Entry B1 from file B"
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
