use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_latest_version_changes() {
  let doing = DoingCmd::new();

  let output = doing.run(["changes"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should contain a version header
  assert!(
    stdout.contains("[v") || stdout.contains("__"),
    "expected version header in output, got: {stdout}"
  );

  // Should contain change entries (lines starting with - or indented -)
  assert!(
    stdout.contains("- "),
    "expected change entries in output, got: {stdout}"
  );
}

#[test]
#[ignore = "changelog alias not implemented (see #200)"]
fn it_is_accessible_via_changelog_alias() {
  let doing = DoingCmd::new();

  let changes_output = doing.run(["changes"]).output().expect("failed to run changes");
  let changelog_output = doing.run(["changelog"]).output().expect("failed to run changelog");

  let changes_stdout = String::from_utf8_lossy(&changes_output.stdout);
  let changelog_stdout = String::from_utf8_lossy(&changelog_output.stdout);

  assert_eq!(
    changes_stdout, changelog_stdout,
    "expected 'changelog' alias to produce same output as 'changes'"
  );
}
