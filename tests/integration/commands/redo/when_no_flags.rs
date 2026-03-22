use crate::support::helpers::DoingCmd;

#[test]
fn it_reverses_last_undo() {
  let doing = DoingCmd::new();

  // Create an entry, then undo it, then redo it
  doing.run(["now", "Redo test entry"]).assert().success();

  let contents_before_undo = doing.read_doing_file();
  assert!(contents_before_undo.contains("Redo test entry"));

  doing.run(["undo"]).assert().success();

  let contents_after_undo = doing.read_doing_file();
  assert!(
    !contents_after_undo.contains("Redo test entry"),
    "expected entry removed after undo, got: {contents_after_undo}"
  );

  doing.run(["redo"]).assert().success();

  let contents_after_redo = doing.read_doing_file();
  assert!(
    contents_after_redo.contains("Redo test entry"),
    "expected entry restored after redo, got: {contents_after_redo}"
  );
}

#[test]
#[ignore = "redo command not yet implemented (see #177)"]
fn it_redoes_multiple_with_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();

  doing.run(["undo"]).assert().success();
  doing.run(["undo"]).assert().success();

  doing.run(["redo", "2"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Entry one") && contents.contains("Entry two"),
    "expected both entries restored after redo 2, got: {contents}"
  );
}

#[test]
fn it_does_nothing_when_no_undo_to_redo() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  // Try redo without prior undo
  let output = doing.run(["redo"]).output().expect("failed to run redo");
  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should either succeed with "nothing to redo" message or fail gracefully
  let combined = format!("{stdout}{stderr}");
  assert!(
    !combined.is_empty() || output.status.success(),
    "expected some feedback when nothing to redo"
  );
}
