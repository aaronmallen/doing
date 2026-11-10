use crate::support::helpers::DoingCmd;

#[test]
fn it_orders_tags() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1h", "Tag order @project"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["since", "2h ago", "--totals", "--tag-order", "asc"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
