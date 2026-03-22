use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_help_for_specified_command() {
  let doing = DoingCmd::new();

  let output = doing.run(["help", "now"]).output().expect("failed to run help now");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected help now to exit successfully");
  assert!(
    stdout.contains("now") || stdout.contains("Add"),
    "expected help now to show now usage, got: {stdout}"
  );
}

#[test]
fn it_shows_help_for_multiple_commands() {
  let doing = DoingCmd::new();

  let output = doing.run(["help", "show"]).output().expect("failed to run help show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected help show to exit successfully");
  assert!(
    stdout.contains("show") || stdout.contains("Show"),
    "expected help show to show show usage, got: {stdout}"
  );
}
