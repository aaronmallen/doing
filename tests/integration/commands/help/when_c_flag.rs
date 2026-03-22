use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "--compact flag not implemented (see DEV-0007)"]
fn it_lists_commands_in_compact_format() {
  let doing = DoingCmd::new();

  let output = doing.run(["help", "-c"]).output().expect("failed to run help -c");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected help -c to exit successfully");
  // Compact format should list command names without full descriptions
  assert!(
    stdout.contains("now") && stdout.contains("done"),
    "expected compact help to list command names, got: {stdout}"
  );
}
