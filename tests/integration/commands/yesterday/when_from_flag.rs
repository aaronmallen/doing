use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_time_range() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday from test"])
    .assert()
    .success();

  let output = doing
    .run(["yesterday", "--from", "6am to 11pm"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
