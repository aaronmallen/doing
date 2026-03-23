use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_time_intervals() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing.run(["now", "--back", "1h", "Completed task"]).assert().success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["on", &today, "--times", "--template", "%title %interval"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Completed task"),
    "expected entry in output with --times, got: {stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    re.is_match(&stdout),
    "expected time interval in HH:MM:SS format in output with --times, got: {stdout}"
  );
}

#[test]
fn it_shows_with_short_flag() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing.run(["now", "--back", "1h", "Completed task"]).assert().success();
  doing.run(["done"]).assert().success();

  let long_output = doing
    .run(["on", &today, "--times", "--template", "%title %interval"])
    .output()
    .expect("failed to run");
  let short_output = doing
    .run(["on", &today, "-t", "--template", "%title %interval"])
    .output()
    .expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -t to produce same output as --times"
  );
}

#[test]
fn it_shows_time_info_not_present_without_flag() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing.run(["now", "--back", "1h", "No times task"]).assert().success();
  doing.run(["done"]).assert().success();

  let with_times = doing
    .run(["on", &today, "--times", "--template", "%title %interval"])
    .output()
    .expect("failed to run");
  let without_times = doing
    .run(["on", &today, "--template", "%title"])
    .output()
    .expect("failed to run");

  let with_stdout = String::from_utf8_lossy(&with_times.stdout);
  let without_stdout = String::from_utf8_lossy(&without_times.stdout);

  assert!(
    with_stdout.contains("No times task"),
    "expected entry in --times output, got: {with_stdout}"
  );
  assert!(
    without_stdout.contains("No times task"),
    "expected entry in output without --times, got: {without_stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    re.is_match(&with_stdout),
    "expected interval in --times output, got: {with_stdout}"
  );
  assert!(
    !re.is_match(&without_stdout),
    "expected no interval in title-only template output, got: {without_stdout}"
  );
}
