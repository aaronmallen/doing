use crate::support::helpers::DoingCmd;

#[test]
fn it_sorts_by_tag() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "27h", "First task @zebra"])
    .assert()
    .success();
  doing.run(["done", "--back", "26h"]).assert().success();

  doing
    .run(["now", "--back", "26h", "Second task @alpha"])
    .assert()
    .success();
  doing.run(["done", "--back", "25h"]).assert().success();

  doing
    .run(["now", "--back", "25h", "Third task @middle"])
    .assert()
    .success();
  doing.run(["done", "--back", "24h"]).assert().success();

  let output = doing
    .run(["yesterday", "--totals", "--tag-sort", "name"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("alpha"), "expected @alpha in output, got: {stdout}");
  assert!(stdout.contains("middle"), "expected @middle in output, got: {stdout}");
  assert!(stdout.contains("zebra"), "expected @zebra in output, got: {stdout}");
}
