use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "--prune flag not yet implemented (see #175)"]
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
#[ignore = "--prune flag not yet implemented (see #175)"]
fn it_prunes_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  let output = doing.run(["undo", "-p"]).output().expect("failed to run undo -p");

  assert!(
    output.status.success(),
    "expected undo -p to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
