use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Currently entry"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let output = doing
    .run(["on", &today, "--section", "Currently"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently entry"),
    "expected entry from Currently section, got: {stdout}"
  );
}

#[test]
fn it_limits_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Currently entry"]).assert().success();

  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  let long_output = doing
    .run(["on", &today, "--section", "Currently"])
    .output()
    .expect("failed to run");
  let short_output = doing
    .run(["on", &today, "-s", "Currently"])
    .output()
    .expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -s to produce same output as --section"
  );
}
