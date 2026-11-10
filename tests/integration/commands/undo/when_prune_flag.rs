use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_old_backups() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();

  let output = doing
    .run(["undo", "--prune"])
    .output()
    .expect("failed to run undo --prune");

  assert!(
    output.status.success(),
    "expected undo --prune to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn it_prunes_with_long_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing
    .run(["undo", "--prune"])
    .output()
    .expect("failed to run undo --prune");

  assert!(
    output.status.success(),
    "expected undo --prune to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
