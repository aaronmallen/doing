use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_default_selection_without_prompting() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry to finish with default"]).assert().success();

  // --default should select the default option without prompting
  doing.run(["--default", "finish"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected entry to be finished with --default, got: {contents}"
  );
}
