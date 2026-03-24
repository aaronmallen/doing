use crate::support::helpers::DoingCmd;

#[test]
fn it_orders_tags() {
  let doing = DoingCmd::new();

  // Use absolute times firmly within today to avoid midnight timezone flakes on CI.
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", &format!("{today} 10:00"), "First task @zebra"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 11:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{today} 11:00"), "Second task @alpha"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 12:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{today} 12:00"), "Third task @middle"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 13:00")])
    .assert()
    .success();

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

  // Use absolute times firmly within today to avoid midnight timezone flakes on CI.
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", &format!("{today} 10:00"), "First task @zebra"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 11:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{today} 11:00"), "Second task @alpha"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 12:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{today} 12:00"), "Third task @middle"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 13:00")])
    .assert()
    .success();

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
