use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_changes_by_search() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--search", "tag"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Every change entry line should contain "tag" (case-insensitive)
  let entry_lines: Vec<&str> = stdout
    .lines()
    .filter(|line| line.trim_start().starts_with("- ") || line.trim_start().starts_with("("))
    .collect();

  assert!(
    !entry_lines.is_empty(),
    "expected at least one matching change entry for search 'tag', got: {stdout}"
  );

  for line in &entry_lines {
    assert!(
      line.to_lowercase().contains("tag"),
      "expected every change entry to match search 'tag', but found: {line}"
    );
  }
}

#[test]
fn it_filters_with_short_flag() {
  let doing = DoingCmd::new();

  let long_output = doing
    .run(["changes", "--all", "--search", "tag"])
    .output()
    .expect("failed to run");
  let short_output = doing
    .run(["changes", "--all", "-s", "tag"])
    .output()
    .expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -s to produce same output as --search"
  );
}
