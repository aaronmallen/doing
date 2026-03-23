use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_error_for_unknown_command() {
  let doing = DoingCmd::new();
  let output = doing.run(["notacommand"]).output().expect("failed to run doing");

  assert!(!output.status.success());
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(!stderr.is_empty(), "expected error message on stderr");
}

#[test]
fn it_shows_error_for_unknown_flag() {
  let doing = DoingCmd::new();
  let output = doing.run(["--notaflag"]).output().expect("failed to run doing");

  assert!(!output.status.success());
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(!stderr.is_empty(), "expected error message on stderr");
}

#[test]
fn it_suggests_similar_command() {
  let doing = DoingCmd::new();
  let output = doing.run(["shwo"]).output().expect("failed to run doing");

  assert!(!output.status.success());
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("show"),
    "expected suggestion for 'show' in stderr, got: {stderr}"
  );
}
