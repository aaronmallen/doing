use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_time_intervals() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["done", "--back", &format!("{today} 09:00"), "Completed grep task"])
    .assert()
    .success();

  let output = doing
    .run(["grep", "Completed grep", "--times"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Completed grep task"),
    "expected entry in output with --times, got: {stdout}"
  );
}

#[test]
fn it_shows_with_short_flag() {
  let doing = DoingCmd::new();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["done", "--back", &format!("{today} 09:00"), "Completed grep task"])
    .assert()
    .success();

  let long_output = doing
    .run(["grep", "Completed grep", "--times"])
    .output()
    .expect("failed to run");
  let short_output = doing
    .run(["grep", "Completed grep", "-t"])
    .output()
    .expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -t to produce same output as --times"
  );
}
