use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_duration_on_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on stuff"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing.run(["on", &today, "--duration"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Working on stuff"),
    "expected entry in output with --duration, got: {stdout}"
  );
}

#[test]
fn it_includes_interval_for_finished_entries() {
  let doing = DoingCmd::new();

  // Use the backdated time's date for the `on` query to avoid flakes near midnight.
  let back_time = chrono::Local::now() - chrono::Duration::hours(1);
  let entry_date = back_time.format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--back", "1h", "Finished interval entry"])
    .assert()
    .success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["on", &entry_date, "--duration", "--template", "%title %interval"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Finished interval entry"),
    "expected entry in output, got: {stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    re.is_match(&stdout),
    "expected interval in HH:MM:SS format in output, got: {stdout}"
  );
}
