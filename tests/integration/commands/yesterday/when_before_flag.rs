use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_before_time() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday entry"]).assert().success();

  let output = doing
    .run(["yesterday", "--before", "11pm"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
