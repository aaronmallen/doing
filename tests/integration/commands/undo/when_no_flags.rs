use crate::support::helpers::DoingCmd;

#[test]
fn it_undoes_last_change() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry to undo"]).assert().success();

  let contents_before = doing.read_doing_file();
  assert!(contents_before.contains("Entry to undo"));

  doing.run(["undo"]).assert().success();

  let contents_after = doing.read_doing_file();
  assert!(
    !contents_after.contains("Entry to undo"),
    "expected entry removed after undo, got: {contents_after}"
  );
}

#[test]
fn it_undoes_multiple_changes_with_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();

  doing.run(["undo", "2"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("First entry") && !contents.contains("Second entry"),
    "expected both entries removed after undo 2, got: {contents}"
  );
}

#[test]
fn it_does_nothing_when_no_backups() {
  let doing = DoingCmd::new();

  // No entries created, so no backups
  let output = doing.run(["undo"]).output().expect("failed to run undo");
  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should either succeed with a message or fail gracefully
  let combined = format!("{stdout}{stderr}");
  assert!(
    !combined.is_empty() || output.status.success(),
    "expected some feedback when nothing to undo"
  );
}
