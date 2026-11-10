use crate::support::helpers::DoingCmd;

#[test]
fn it_sorts_by_tag() {
  let doing = DoingCmd::new();

  // Use absolute times on yesterday to avoid midnight timezone flakes on CI.
  let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
    .format("%Y-%m-%d")
    .to_string();

  doing
    .run(["now", "--from", &format!("{yesterday} 10:00"), "First task @zebra"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{yesterday} 11:00"), "Second task @alpha"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 12:00")])
    .assert()
    .success();

  doing
    .run(["now", "--from", &format!("{yesterday} 12:00"), "Third task @middle"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 13:00")])
    .assert()
    .success();

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
