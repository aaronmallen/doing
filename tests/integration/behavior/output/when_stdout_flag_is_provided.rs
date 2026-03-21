use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "requires --stdout flag implementation (see #155)"]
fn it_redirects_info_messages_to_stdout() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["--stdout", "now", "Stdout redirect test"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(!stdout.is_empty(), "expected info on stdout with --stdout, got empty");
  assert!(stderr.is_empty(), "expected empty stderr with --stdout, got: {stderr}");
}

#[test]
fn it_sends_display_output_to_stdout_as_usual() {
  let doing = DoingCmd::new();
  doing.run(["now", "Stdout display test"]).assert().success();

  let output = doing.run(["--stdout", "show"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Stdout display test"),
    "expected entry on stdout with --stdout show, got: {stdout}"
  );
}

#[test]
fn it_combines_info_and_display_on_stdout_for_commands_that_do_both() {
  let doing = DoingCmd::new();
  doing.run(["now", "Combined output test"]).assert().success();

  let output = doing.run(["--stdout", "last"]).output().expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    !stdout.is_empty(),
    "expected output on stdout with --stdout last, got empty"
  );
  assert!(
    stderr.is_empty(),
    "expected empty stderr with --stdout last, got: {stderr}"
  );
}
