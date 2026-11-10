use crate::support::helpers::DoingCmd;

#[test]
fn it_undoes_the_last_undo() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["undo"]).assert().success();
  doing.run(["undo", "--redo"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Test entry"),
    "expected entry restored with --redo, got: {contents}"
  );
}

#[test]
fn it_undoes_the_last_undo_with_long_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["undo"]).assert().success();
  doing.run(["undo", "--redo"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Test entry"),
    "expected entry restored with --redo, got: {contents}"
  );
}

#[test]
fn it_disables_redo_with_no_redo() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["undo", "--no-redo"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Test entry"),
    "expected entry removed with --no-redo, got: {contents}"
  );
}
