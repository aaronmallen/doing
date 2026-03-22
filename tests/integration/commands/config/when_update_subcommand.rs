use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "config update/refresh not implemented (see DEV-0005)"]
fn it_refreshes_config() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "update"])
    .output()
    .expect("failed to run config update");

  assert!(
    output.status.success(),
    "expected config update to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
#[ignore = "config update/refresh not implemented (see DEV-0005)"]
fn it_refreshes_with_refresh_alias() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["config", "refresh"])
    .output()
    .expect("failed to run config refresh");

  assert!(
    output.status.success(),
    "expected config refresh to succeed, stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
