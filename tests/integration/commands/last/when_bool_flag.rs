use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_and_boolean_for_multiple_filters() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @alpha"]).assert().success();
  doing.run(["now", "Entry @alpha @beta"]).assert().success();

  let output = doing
    .run(["last", "--tag", "alpha,beta", "--bool", "AND"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry @alpha @beta"),
    "expected entry with both tags, got: {stdout}"
  );
}

#[test]
fn it_uses_or_boolean_for_multiple_filters() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry no tags"]).assert().success();
  doing.run(["now", "Entry @alpha"]).assert().success();

  let output = doing
    .run(["last", "--tag", "alpha,beta", "--bool", "OR"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry @alpha"),
    "expected entry with matching tag, got: {stdout}"
  );
}

#[test]
fn it_uses_not_boolean_for_multiple_filters() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @alpha"]).assert().success();
  doing.run(["now", "Entry no tags"]).assert().success();

  let output = doing
    .run(["last", "--tag", "alpha", "--bool", "NOT"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry no tags"),
    "expected non-tagged entry with NOT, got: {stdout}"
  );
}
