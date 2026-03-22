use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_exact_matching() {
  let doing = DoingCmd::new();

  doing.run(["now", "Exact match phrase here"]).assert().success();
  doing.run(["now", "Other entry"]).assert().success();

  let output = doing
    .run(["grep", "Exact match phrase", "--exact"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Exact match phrase"),
    "expected exact-matching entry, got: {stdout}"
  );
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Exact short test"]).assert().success();

  let long_output = doing
    .run(["grep", "Exact short", "--exact"])
    .output()
    .expect("failed to run");
  let short_output = doing
    .run(["grep", "Exact short", "-x"])
    .output()
    .expect("failed to run");

  let long_stdout = String::from_utf8_lossy(&long_output.stdout);
  let short_stdout = String::from_utf8_lossy(&short_output.stdout);

  assert_eq!(
    long_stdout, short_stdout,
    "expected -x to produce same output as --exact"
  );
}
