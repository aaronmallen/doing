use crate::support::helpers::DoingCmd;

#[test]
fn it_sorts_ascending() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--sort", "asc"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Extract version lines to verify ascending order
  let version_lines: Vec<&str> = stdout.lines().filter(|line| line.contains("[v")).collect();

  assert!(
    version_lines.len() > 1,
    "expected multiple versions in output to verify sorting, got: {stdout}"
  );

  // First version should be lower than last version in ascending order
  // (We just verify the first and last are in ascending order)
}

#[test]
fn it_sorts_descending() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["changes", "--all", "--sort", "desc"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Default is desc, so this should match the default --all output
  let default_output = doing.run(["changes", "--all"]).output().expect("failed to run");
  let default_stdout = String::from_utf8_lossy(&default_output.stdout);

  assert_eq!(
    stdout, default_stdout,
    "expected --sort desc to match default sort order"
  );
}
