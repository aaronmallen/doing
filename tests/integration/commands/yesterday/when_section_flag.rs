use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_section() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday Currently"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1d", "--section", "Later", "Yesterday Later"])
    .assert()
    .success();

  let output = doing
    .run(["yesterday", "--section", "Currently"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday Currently"),
    "expected entry from Currently, got: {stdout}"
  );
  assert!(
    !stdout.contains("Yesterday Later"),
    "expected no entry from Later, got: {stdout}"
  );
}

#[test]
fn it_limits_with_short_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday Currently"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1d", "--section", "Later", "Yesterday Later"])
    .assert()
    .success();

  let output = doing
    .run(["yesterday", "-s", "Currently"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday Currently"),
    "expected entry from Currently, got: {stdout}"
  );
}
