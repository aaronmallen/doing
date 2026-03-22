use crate::support::helpers::DoingCmd;

#[test]
fn it_sorts_by_tag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "25h", "Yesterday tag sort @project"])
    .assert()
    .success();
  doing.run(["done", "--back", "24h"]).assert().success();

  let output = doing
    .run(["yesterday", "--totals", "--tag-sort", "name"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
