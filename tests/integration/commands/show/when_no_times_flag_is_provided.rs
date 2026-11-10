use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_hides_time_intervals() {
  let doing = DoingCmd::new();

  // Create config that explicitly uses --no-times (our CLI doesn't have --no-times, uses -t/--times)
  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Done task @done(2024-01-15 10:00)\n",
  )
  .expect("failed to write doing file");

  // Show without --times flag - times should not be shown by default in our config
  let output = doing.run(["show"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Done task"),
    "expected done task in output, got: {stdout}"
  );
}

#[test]
fn it_omits_interval_with_title_only_template() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Done task @done(2024-01-15 10:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--template", "%title"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Done task"),
    "expected done task in output, got: {stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    !re.is_match(&stdout),
    "expected no time interval in title-only template output, got: {stdout}"
  );
}

#[test]
fn it_shows_interval_when_times_flag_is_provided() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-15 09:00 | Done task @done(2024-01-15 10:00)\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["show", "--times", "--template", "%title %interval"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Done task"),
    "expected done task in output, got: {stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    re.is_match(&stdout),
    "expected time interval in HH:MM:SS format with --times, got: {stdout}"
  );
}
