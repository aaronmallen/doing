use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_changes_only_without_headers() {
  let doing = DoingCmd::new();

  let output = doing.run(["changes", "-C"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // With -C, should NOT contain version headers or section headers
  assert!(
    !stdout.contains("[v"),
    "expected no version headers with -C flag, got: {stdout}"
  );

  // Should still contain change text
  assert!(
    !stdout.trim().is_empty(),
    "expected some change content with -C flag, got empty output"
  );
}

#[test]
fn it_shows_with_long_flag() {
  let doing = DoingCmd::new();

  let short_output = doing.run(["changes", "-C"]).output().expect("failed to run");
  let long_output = doing.run(["changes", "--changes"]).output().expect("failed to run");

  let short_stdout = String::from_utf8_lossy(&short_output.stdout);
  let long_stdout = String::from_utf8_lossy(&long_output.stdout);

  assert_eq!(
    short_stdout, long_stdout,
    "expected --changes to produce same output as -C"
  );
}
