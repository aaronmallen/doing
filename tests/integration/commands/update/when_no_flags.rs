use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "update is self-update, not config update (see DEV-0006)"]
fn it_accepts_update_command() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["update", "--help"])
    .output()
    .expect("failed to run update --help");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected update --help to succeed");
  assert!(
    stdout.contains("update") || stdout.contains("Update"),
    "expected help text to mention update, got: {stdout}"
  );
}
