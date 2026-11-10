use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  // Use absolute times firmly within today to avoid midnight timezone flakes on CI.
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  // Create a timed entry (started at 10:00, done at 11:00)
  doing
    .run(["now", "--from", &format!("{today} 10:00"), "Timed entry"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 11:00")])
    .assert()
    .success();

  // Create an open entry (no @done, so no time interval)
  doing.run(["now", "Open entry"]).assert().success();

  let output = doing.run(["today", "--only-timed"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Timed entry"),
    "expected timed entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Open entry"),
    "expected open entry to be excluded, got: {stdout}"
  );
}
