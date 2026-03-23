use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_after_time() {
  let doing = DoingCmd::new();

  // Create two entries at different times yesterday
  doing
    .run(["now", "--back", "30h", "Early yesterday entry"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "25h", "Late yesterday entry"])
    .assert()
    .success();

  // Use a relative time that falls between the two entries
  let output = doing
    .run(["yesterday", "--after", "26h ago"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Late yesterday entry"),
    "expected late entry in output, got: {stdout}"
  );
  assert!(
    !stdout.contains("Early yesterday entry"),
    "expected early entry to be excluded by --after filter, got: {stdout}"
  );
}
