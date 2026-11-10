use crate::support::helpers::DoingCmd;

#[test]
fn it_auto_creates_file_on_write() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(!contents.is_empty(), "expected doing file to be created");
}

#[test]
fn it_creates_file_with_section_header() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Currently"),
    "expected 'Currently' section header, got: {contents}"
  );
}

#[test]
fn it_handles_show_on_missing_file_gracefully() {
  let doing = DoingCmd::new();

  let output = doing.run(["show"]).output().expect("failed to run doing");

  assert!(output.status.success(), "expected show to succeed on missing file");
}
