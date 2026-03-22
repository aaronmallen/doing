use crate::support::helpers::DoingCmd;

#[test]
fn it_orders_tags() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "25h", "Yesterday tag order @project"])
    .assert()
    .success();
  doing.run(["done", "--back", "24h"]).assert().success();

  let output = doing
    .run(["yesterday", "--totals", "--tag-order", "asc"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
