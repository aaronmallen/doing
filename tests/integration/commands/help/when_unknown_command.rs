use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_error_for_unknown() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["help", "notacommand"])
    .output()
    .expect("failed to run help notacommand");
  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);
  let combined = format!("{stdout}{stderr}");

  // Should indicate the command is unknown
  assert!(
    combined.contains("unrecognized") || combined.contains("Unknown") || combined.contains("error"),
    "expected error about unknown command, got stdout: {stdout}, stderr: {stderr}"
  );
}
