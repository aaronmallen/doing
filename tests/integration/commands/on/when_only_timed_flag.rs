use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_only_timed_entries() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  // Create a done entry with an explicit duration so the interval is always > 0
  doing
    .run([
      "done",
      "--back",
      &format!("{today} 09:00"),
      "--took",
      "1h",
      "Finished task",
    ])
    .assert()
    .success();

  // Create an open entry (no time interval)
  doing.run(["now", "Open task"]).assert().success();

  let output = doing
    .run(["on", &today, "--only-timed"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Finished task"),
    "expected timed entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Open task"),
    "expected non-timed entry excluded with --only-timed, got: {stdout}"
  );
}
