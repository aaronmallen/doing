use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_after_time() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday entry"]).assert().success();

  let output = doing
    .run(["yesterday", "--after", "6am"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
