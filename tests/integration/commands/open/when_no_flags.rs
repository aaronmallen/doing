use crate::support::helpers::DoingCmd;

#[test]
fn it_opens_doing_file_in_default_editor() {
  // The test config sets editors.default to "cat", which will just output the file contents
  let doing = DoingCmd::new();

  // Create an entry first so the doing file exists
  doing.run(["now", "Test entry"]).assert().success();

  // `doing open` with editor set to "cat" should succeed and output the file
  let output = doing.run(["open"]).output().expect("failed to run open");

  assert!(
    output.status.success(),
    "expected open to exit successfully, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
