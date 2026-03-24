use crate::support::helpers::DoingCmd;

#[test]
fn it_sorts_by_tag() {
  let doing = DoingCmd::new();

  // Use absolute times firmly within today to avoid midnight timezone flakes on CI.
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", &format!("{today} 10:00"), "First task @zebra"])
    .assert()
    .success();
  doing
    .run(["done", "--from", &format!("{today} 11:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{today} 11:00"), "Second task @alpha"])
    .assert()
    .success();
  doing
    .run(["done", "--from", &format!("{today} 12:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{today} 12:00"), "Third task @middle"])
    .assert()
    .success();
  doing
    .run(["done", "--from", &format!("{today} 13:00")])
    .assert()
    .success();

  let output = doing
    .run(["today", "--totals", "--tag-sort", "name"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("alpha"), "expected @alpha in output, got: {stdout}");
  assert!(stdout.contains("middle"), "expected @middle in output, got: {stdout}");
  assert!(stdout.contains("zebra"), "expected @zebra in output, got: {stdout}");
}
