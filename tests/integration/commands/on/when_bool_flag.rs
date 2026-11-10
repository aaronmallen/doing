use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_and_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @project @urgent"]).assert().success();
  doing.run(["now", "Entry @project only"]).assert().success();
  doing.run(["now", "Entry @urgent only"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--tag", "project,urgent", "--bool", "AND"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry @project @urgent"),
    "expected entry with both tags, got: {stdout}"
  );
  // With AND, entries with only one of the tags should be excluded
  assert!(
    !stdout.contains("Entry @project only"),
    "expected entry with only @project excluded with AND, got: {stdout}"
  );
}

#[test]
fn it_uses_or_boolean() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry @project"]).assert().success();
  doing.run(["now", "Entry @urgent"]).assert().success();
  doing.run(["now", "Entry no tags"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--tag", "project,urgent", "--bool", "OR"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Entry @project"),
    "expected entry with @project tag, got: {stdout}"
  );
  assert!(
    stdout.contains("Entry @urgent"),
    "expected entry with @urgent tag, got: {stdout}"
  );
  assert!(
    !stdout.contains("Entry no tags"),
    "expected entry without tags excluded with OR, got: {stdout}"
  );
}
