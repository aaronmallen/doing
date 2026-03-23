use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_in_column_format() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "--column", "--list"])
    .output()
    .expect("failed to run template --column --list");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected template --column to succeed");
  assert!(!stdout.is_empty(), "expected column output");
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "-c", "--list"])
    .output()
    .expect("failed to run template -c --list");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected template -c to succeed");
  assert!(!stdout.is_empty(), "expected column output");
}

#[test]
fn it_disables_column_format() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "--no-column", "--list"])
    .output()
    .expect("failed to run template --no-column --list");

  assert!(
    output.status.success(),
    "expected template --no-column to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
