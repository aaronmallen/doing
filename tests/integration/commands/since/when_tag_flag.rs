use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Tagged entry @project"]).assert().success();
  doing.run(["now", "Untagged entry"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--tag", "project"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Tagged entry"), "expected tagged entry, got: {stdout}");
  assert!(
    !stdout.contains("Untagged entry"),
    "expected no untagged entry, got: {stdout}"
  );
}
