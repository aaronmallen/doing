use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_duration() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday duration"])
    .assert()
    .success();

  let output = doing.run(["yesterday", "--duration"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday duration"),
    "expected entry in output, got: {stdout}"
  );
}

#[test]
fn it_hides_duration() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "1d", "Yesterday no duration"])
    .assert()
    .success();

  let output = doing.run(["yesterday"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday no duration"),
    "expected entry in output, got: {stdout}"
  );
}

#[test]
fn it_includes_interval_for_finished_entries_with_custom_template() {
  let doing = DoingCmd::new();

  // Use absolute times on yesterday to avoid midnight timezone flakes on CI.
  let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
    .format("%Y-%m-%d")
    .to_string();

  doing
    .run([
      "now",
      "--from",
      &format!("{yesterday} 10:00"),
      "Yesterday finished entry",
    ])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  let output = doing
    .run(["yesterday", "--duration", "--template", "%title %interval"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Yesterday finished entry"),
    "expected entry in output, got: {stdout}"
  );

  let re = regex::Regex::new(r"\d+:\d+:\d+").unwrap();
  assert!(
    re.is_match(&stdout),
    "expected interval in HH:MM:SS format in output, got: {stdout}"
  );
}
