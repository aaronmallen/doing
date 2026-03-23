use crate::support::helpers::DoingCmd;

#[test]
fn it_orders_tags() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h", "First task @zebra"])
    .assert()
    .success();
  doing.run(["done", "--back", "2h"]).assert().success();

  doing
    .run(["now", "--back", "2h", "Second task @alpha"])
    .assert()
    .success();
  doing.run(["done", "--back", "1h"]).assert().success();

  doing
    .run(["now", "--back", "1h", "Third task @middle"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["today", "--totals", "--tag-order", "asc"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let totals_section = stdout
    .split("Tag Totals")
    .nth(1)
    .expect("expected Tag Totals section in output");

  let alpha_pos = totals_section.find("alpha").expect("expected alpha in totals");
  let middle_pos = totals_section.find("middle").expect("expected middle in totals");
  let zebra_pos = totals_section.find("zebra").expect("expected zebra in totals");
  assert!(
    alpha_pos < middle_pos && middle_pos < zebra_pos,
    "expected tags in ascending order (alpha < middle < zebra) in totals, got: {totals_section}"
  );
}

#[test]
fn it_orders_tags_descending() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h", "First task @zebra"])
    .assert()
    .success();
  doing.run(["done", "--back", "2h"]).assert().success();

  doing
    .run(["now", "--back", "2h", "Second task @alpha"])
    .assert()
    .success();
  doing.run(["done", "--back", "1h"]).assert().success();

  doing
    .run(["now", "--back", "1h", "Third task @middle"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["today", "--totals", "--tag-order", "desc"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let totals_section = stdout
    .split("Tag Totals")
    .nth(1)
    .expect("expected Tag Totals section in output");

  let alpha_pos = totals_section.find("alpha").expect("expected alpha in totals");
  let middle_pos = totals_section.find("middle").expect("expected middle in totals");
  let zebra_pos = totals_section.find("zebra").expect("expected zebra in totals");
  assert!(
    zebra_pos < middle_pos && middle_pos < alpha_pos,
    "expected tags in descending order (zebra < middle < alpha) in totals, got: {totals_section}"
  );
}
