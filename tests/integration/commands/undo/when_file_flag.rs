use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "--file flag not yet implemented (see #174)"]
fn it_undoes_changes_to_specified_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let file_path = doing.doing_file_path().to_str().unwrap().to_string();
  doing.run(["undo", "--file", &file_path]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Test entry"),
    "expected entry removed with --file flag, got: {contents}"
  );
}

#[test]
#[ignore = "--file flag not yet implemented (see #174)"]
fn it_undoes_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let file_path = doing.doing_file_path().to_str().unwrap().to_string();
  doing.run(["undo", "-f", &file_path]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Test entry"),
    "expected entry removed with -f flag, got: {contents}"
  );
}
