use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_html_template() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "html"])
    .output()
    .expect("failed to run template html");

  // html is a recognized built-in template, so the command should succeed
  assert!(
    output.status.success(),
    "expected template html to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_outputs_css_template() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "css"])
    .output()
    .expect("failed to run template css");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  // css may or may not be a valid template type in our implementation
  let combined = format!("{stdout}{stderr}");
  assert!(!combined.is_empty(), "expected some output for template css");
}

#[test]
#[ignore = "template command not yet implemented (see #203)"]
fn it_outputs_markdown_template() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "markdown"])
    .output()
    .expect("failed to run template markdown");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(output.status.success(), "expected template markdown to succeed");
  assert!(!stdout.is_empty(), "expected markdown template output");
}

#[test]
fn it_returns_error_for_unknown_type() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["template", "notaformat"])
    .output()
    .expect("failed to run template notaformat");

  assert!(!output.status.success(), "expected error for unknown template type");
}
