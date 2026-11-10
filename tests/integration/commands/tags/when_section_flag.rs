use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Current task @active"]).assert().success();
  doing.run(["done", "Finished task @complete"]).assert().success();

  let output = doing
    .run(["tags", "--section", "Currently"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("active"),
    "expected 'active' tag from Currently section, got: {stdout}"
  );
}

#[test]
fn it_limits_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Current task @active"]).assert().success();
  doing.run(["done", "Finished task @complete"]).assert().success();

  let output = doing.run(["tags", "-s", "Currently"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("active"),
    "expected 'active' tag from Currently section with -s, got: {stdout}"
  );
}
