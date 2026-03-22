use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_markdown_format() {
  let doing = DoingCmd::new();

  let output = doing.run(["changes", "--markdown"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Markdown output should contain markdown headers (### or ####)
  assert!(
    stdout.contains("###"),
    "expected markdown headers in output, got: {stdout}"
  );

  // Markdown output should contain list items with "- "
  assert!(
    stdout.contains("- "),
    "expected markdown list items in output, got: {stdout}"
  );
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  let long_output = doing.run(["changes", "--markdown"]).output().expect("failed to run");
  let short_output = doing.run(["changes", "-m"]).output().expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -m to produce same output as --markdown"
  );
}
