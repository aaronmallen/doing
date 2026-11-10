use crate::support::helpers::DoingCmd;

#[test]
fn it_sends_output_directly_to_stdout() {
  let doing = DoingCmd::new();
  doing.run(["now", "No pager test"]).assert().success();

  let output = doing.run(["--no-pager", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("No pager test"),
    "expected output on stdout without pager, got: {stdout}"
  );
}
