use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Tagged entry @project"]).assert().success();
  doing.run(["now", "Untagged entry"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--tag", "project"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Tagged entry"),
    "expected tagged entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Untagged entry"),
    "expected untagged entry excluded from output, got: {stdout}"
  );
}
