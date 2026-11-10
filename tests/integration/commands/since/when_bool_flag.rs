use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_and_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Both tags @project @urgent"]).assert().success();
  doing.run(["now", "One tag @project"]).assert().success();

  let output = doing
    .run([
      "since", "1h ago", "--tag", "project", "--tag", "urgent", "--bool", "and",
    ])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Both tags"),
    "expected entry with both tags, got: {stdout}"
  );
  assert!(
    !stdout.contains("One tag"),
    "expected entry with only one tag excluded, got: {stdout}"
  );
}

#[test]
fn it_uses_or_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Has project @project"]).assert().success();
  doing.run(["now", "Has urgent @urgent"]).assert().success();
  doing.run(["now", "Has neither"]).assert().success();

  let output = doing
    .run(["since", "1h ago", "--tag", "project", "--tag", "urgent", "--bool", "or"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Has project"), "expected @project entry, got: {stdout}");
  assert!(stdout.contains("Has urgent"), "expected @urgent entry, got: {stdout}");
  assert!(
    !stdout.contains("Has neither"),
    "expected untagged entry excluded, got: {stdout}"
  );
}
