use crate::support::helpers::DoingCmd;

#[test]
fn it_redoes_for_specified_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();
  doing.run(["undo"]).assert().success();

  let file_path = doing.doing_file_path().to_str().unwrap().to_string();
  doing.run(["redo", "--file", &file_path]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Test entry"),
    "expected entry restored with --file flag, got: {contents}"
  );
}
