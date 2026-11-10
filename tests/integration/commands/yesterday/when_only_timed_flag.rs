use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  // Use absolute times on yesterday to avoid midnight timezone flakes on CI.
  let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
    .format("%Y-%m-%d")
    .to_string();

  // Create a timed entry from yesterday
  doing
    .run(["now", "--from", &format!("{yesterday} 10:00"), "Yesterday timed"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  // Create an open entry from yesterday (--back 1d is safe; it always lands on yesterday)
  doing.run(["now", "--back", "1d", "Yesterday open"]).assert().success();

  let output = doing
    .run(["yesterday", "--only-timed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday timed"),
    "expected timed entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Yesterday open"),
    "expected open entry to be excluded, got: {stdout}"
  );
}
