use crate::support::helpers::DoingCmd;

#[test]
fn it_renders_empty_for_open_entries() {
  let doing = DoingCmd::new();
  doing.run(["now", "Interval open test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title (%interval)"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Interval open test"))
    .expect("should find entry line");

  // Open entries (not @done) should have empty interval
  assert!(
    line.contains("()"),
    "expected empty interval for open entry, got: {line}"
  );
}

#[test]
fn it_renders_time_interval_for_done_entries() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "30 minutes ago", "Interval done test"])
    .assert()
    .success();
  doing.run(["finish"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title (%interval)"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Interval done test"))
    .expect("should find entry line");

  // Done entries should have a non-empty interval value in parentheses
  // Ruby doing uses DD:HH:MM format, e.g., "00:00:30" for 30 minutes
  assert!(
    !line.contains("()"),
    "expected non-empty interval for done entry, got: {line}"
  );

  // Should contain time-like digits in the parentheses
  assert!(
    regex::Regex::new(r"\(\d+:\d+:\d+\)").unwrap().is_match(line),
    "expected interval in DD:HH:MM format in parentheses, got: {line}"
  );
}
