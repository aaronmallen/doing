use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_to_time_range() {
  let doing = DoingCmd::new();

  // Use absolute times on yesterday to avoid midnight timezone flakes on CI.
  let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
    .format("%Y-%m-%d")
    .to_string();

  // Create two entries at different times yesterday
  doing
    .run(["now", "--from", &format!("{yesterday} 09:00"), "Early yesterday entry"])
    .assert()
    .success();
  doing
    .run(["now", "--from", &format!("{yesterday} 15:00"), "Late yesterday entry"])
    .assert()
    .success();

  // Use a time range that includes only the late entry
  let output = doing
    .run([
      "yesterday",
      "--from",
      &format!("{yesterday} 12:00 to {yesterday} 18:00"),
    ])
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
