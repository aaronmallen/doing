use crate::support::helpers::DoingCmd;

#[test]
fn it_lists_available_templates() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "--list"])
    .output()
    .expect("failed to run template --list");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected template --list to succeed");
  assert!(stdout.contains("html"), "expected 'html' template, got: {stdout}");
  assert!(
    stdout.contains("markdown"),
    "expected 'markdown' template, got: {stdout}"
  );
}

#[test]
fn it_lists_with_short_flag() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "-l"])
    .output()
    .expect("failed to run template -l");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected template -l to succeed");
  assert!(stdout.contains("html"), "expected 'html' template, got: {stdout}");
}
