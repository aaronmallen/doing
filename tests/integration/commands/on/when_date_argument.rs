use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_entries_for_specific_date() {
  let doing = DoingCmd::new();

  // Create an entry backdated to yesterday
  let yesterday = chrono::Local::now() - chrono::Duration::days(1);
  let yesterday_str = yesterday.format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--back", &format!("{yesterday_str} 10:00"), "Yesterday entry"])
    .assert()
    .success();

  // Create an entry for today
  doing.run(["now", "Today entry"]).assert().success();

  // Query entries for yesterday
  let output = doing.run(["on", &yesterday_str]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Yesterday entry"),
    "expected to see yesterday's entry, got: {stdout}"
  );
  assert!(
    !stdout.contains("Today entry"),
    "expected NOT to see today's entry, got: {stdout}"
  );
}

#[test]
fn it_shows_entries_for_relative_day() {
  let doing = DoingCmd::new();

  // Create an entry backdated to 2 days ago
  let two_days_ago = chrono::Local::now() - chrono::Duration::days(2);
  let date_str = two_days_ago.format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--back", &format!("{date_str} 10:00"), "Two days ago entry"])
    .assert()
    .success();

  doing.run(["now", "Today entry"]).assert().success();

  let output = doing.run(["on", "2d"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Two days ago entry"),
    "expected to see entry from 2 days ago, got: {stdout}"
  );
}

#[test]
#[ignore = "date range 'to' does not include end boundary (see #201)"]
fn it_shows_entries_for_date_range_with_to() {
  let doing = DoingCmd::new();

  let three_days_ago = chrono::Local::now() - chrono::Duration::days(3);
  let one_day_ago = chrono::Local::now() - chrono::Duration::days(1);
  let three_days_str = three_days_ago.format("%Y-%m-%d").to_string();
  let one_day_str = one_day_ago.format("%Y-%m-%d").to_string();

  doing
    .run([
      "now",
      "--back",
      &format!("{three_days_str} 10:00"),
      "Three days ago entry",
    ])
    .assert()
    .success();

  doing
    .run(["now", "--back", &format!("{one_day_str} 10:00"), "One day ago entry"])
    .assert()
    .success();

  doing.run(["now", "Today entry"]).assert().success();

  let output = doing
    .run(["on", &format!("{three_days_str} to {one_day_str}")])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Three days ago entry"),
    "expected to see entry from 3 days ago in range, got: {stdout}"
  );
  assert!(
    stdout.contains("One day ago entry"),
    "expected to see entry from 1 day ago in range, got: {stdout}"
  );
}

#[test]
fn it_shows_nothing_when_no_entries_on_date() {
  let doing = DoingCmd::new();

  // Create an entry for today
  doing.run(["now", "Today entry"]).assert().success();

  // Query for a date far in the past
  let output = doing.run(["on", "2020-01-01"]).output().expect("failed to run");

  let stdout = String::from_utf8_lossy(&output.stdout);

  // Should have no entry output
  assert!(
    !stdout.contains("Today entry"),
    "expected no entries for 2020-01-01, got: {stdout}"
  );
}
