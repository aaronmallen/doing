use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_exact_matching() {
  let doing = DoingCmd::new();

  doing.run(["now", "Project meeting"]).assert().success();
  doing.run(["now", "Meeting project"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--search", "Project meeting", "--exact"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Project meeting"),
    "expected exact-matching entry, got: {stdout}"
  );
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Project meeting"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let long_output = doing
    .run(["on", &today, "--search", "Project meeting", "--exact"])
    .output()
    .expect("failed to run");
  let short_output = doing
    .run(["on", &today, "--search", "Project meeting", "-x"])
    .output()
    .expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -x to produce same output as --exact"
  );
}
