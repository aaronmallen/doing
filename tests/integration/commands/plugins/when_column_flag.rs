use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "plugins command not yet implemented (see #191)"]
fn it_outputs_in_column_format() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["plugins", "--column"])
    .output()
    .expect("failed to run plugins --column");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected plugins --column to succeed");
  assert!(!stdout.is_empty(), "expected column output to be non-empty");
}

#[test]
#[ignore = "plugins command not yet implemented (see #191)"]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing.run(["plugins", "-c"]).output().expect("failed to run plugins -c");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected plugins -c to succeed");
  assert!(!stdout.is_empty(), "expected column output to be non-empty");
}
