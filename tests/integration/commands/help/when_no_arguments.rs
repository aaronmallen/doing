use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_general_help() {
  let doing = DoingCmd::new();

  let output = doing.run(["help"]).output().expect("failed to run help");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected help to exit successfully");
  // Should list available subcommands
  assert!(
    stdout.contains("now"),
    "expected help to list 'now' command, got: {stdout}"
  );
  assert!(
    stdout.contains("done"),
    "expected help to list 'done' command, got: {stdout}"
  );
}
