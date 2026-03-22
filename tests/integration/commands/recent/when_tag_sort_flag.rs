use crate::support::helpers::DoingCmd;

#[test]
fn it_sorts_by_tag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1h", "Tag sort @project"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["recent", "--totals", "--tag-sort", "name"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
