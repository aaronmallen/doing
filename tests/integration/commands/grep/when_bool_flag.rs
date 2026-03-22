use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_and_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @alpha @beta"]).assert().success();
  doing.run(["now", "Entry @alpha only"]).assert().success();
  doing.run(["now", "Entry no tags"]).assert().success();

  let output = doing
    .run(["grep", "Entry", "--tag", "alpha,beta", "--bool", "AND"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry @alpha @beta"),
    "expected entry with both tags, got: {stdout}"
  );
  assert!(
    !stdout.contains("Entry @alpha only"),
    "expected entry with single tag excluded with AND, got: {stdout}"
  );
}

#[test]
fn it_uses_or_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @alpha"]).assert().success();
  doing.run(["now", "Entry @beta"]).assert().success();
  doing.run(["now", "Entry no tags"]).assert().success();

  let output = doing
    .run(["grep", "Entry", "--tag", "alpha,beta", "--bool", "OR"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry @alpha"),
    "expected entry with @alpha tag, got: {stdout}"
  );
  assert!(
    stdout.contains("Entry @beta"),
    "expected entry with @beta tag, got: {stdout}"
  );
  assert!(
    !stdout.contains("Entry no tags"),
    "expected untagged entry excluded with OR, got: {stdout}"
  );
}

#[test]
fn it_uses_not_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @alpha"]).assert().success();
  doing.run(["now", "Entry no tags"]).assert().success();

  let output = doing
    .run(["grep", "Entry", "--tag", "alpha", "--bool", "NOT"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry no tags"),
    "expected untagged entry with NOT, got: {stdout}"
  );
  assert!(
    !stdout.contains("Entry @alpha"),
    "expected tagged entry excluded with NOT, got: {stdout}"
  );
}
