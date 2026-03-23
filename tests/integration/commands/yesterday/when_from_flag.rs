use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_time_range() {
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

  // Use a relative time range that includes only the late entry
  let output = doing
    .run(["yesterday", "--from", "26h ago to 24h ago"])
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
    "expected early entry to be excluded by --from time range, got: {stdout}"
  );
}
