use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry in Currently"]).assert().success();
  doing
    .run(["now", "--section", "Later", "Entry in Later"])
    .assert()
    .success();

  let output = doing
    .run(["since", "1h ago", "--section", "Currently"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Entry in Currently"),
    "expected entry from Currently, got: {stdout}"
  );
  assert!(
    !stdout.contains("Entry in Later"),
    "expected no entry from Later, got: {stdout}"
  );
}

#[test]
fn it_limits_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry in Currently"]).assert().success();
  doing
    .run(["now", "--section", "Later", "Entry in Later"])
    .assert()
    .success();

  let output = doing
    .run(["since", "1h ago", "-s", "Currently"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Entry in Currently"),
    "expected entry from Currently, got: {stdout}"
  );
}
