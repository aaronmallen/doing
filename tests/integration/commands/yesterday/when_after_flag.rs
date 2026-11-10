use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_entries_after_time() {
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

  // Filter to entries after noon yesterday
  let output = doing
    .run(["yesterday", "--after", &format!("{yesterday} 12:00")])
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
