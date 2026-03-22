use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_elapsed_on_unfinished_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Open duration task"]).assert().success();

  let output = doing.run(["show", "--duration"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Open duration task"),
    "expected entry in duration output, got: {stdout}"
  );
}
