use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "update is self-update, not config update (see DEV-0006)"]
fn it_accepts_pre_flag() {
  let doing = DoingCmd::new();

  // --pre should be recognized as a valid flag for the update command
  let output = doing
    .run(["update", "--pre"])
    .output()
    .expect("failed to run update --pre");

  assert!(
    output.status.success(),
    "expected update --pre to be recognized, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
#[ignore = "update is self-update, not config update (see DEV-0006)"]
fn it_accepts_beta_alias() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["update", "--beta"])
    .output()
    .expect("failed to run update --beta");

  assert!(
    output.status.success(),
    "expected update --beta to be recognized, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
#[ignore = "update is self-update, not config update (see DEV-0006)"]
fn it_accepts_short_flag() {
  let doing = DoingCmd::new();

  let output = doing.run(["update", "-p"]).output().expect("failed to run update -p");

  assert!(
    output.status.success(),
    "expected update -p to be recognized, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
