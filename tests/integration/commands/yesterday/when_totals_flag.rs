use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_tag_time_totals() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "25h", "Yesterday tagged @project"])
    .assert()
    .success();
  doing.run(["done", "--back", "24h"]).assert().success();

  let output = doing.run(["yesterday", "--totals"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
