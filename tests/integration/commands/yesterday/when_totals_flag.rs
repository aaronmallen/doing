use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_tag_time_totals() {
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
      "Yesterday tagged @project",
    ])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  let output = doing.run(["yesterday", "--totals"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Yesterday tagged"),
    "expected entry title in output, got: {stdout}"
  );

  assert!(
    stdout.contains("project"),
    "expected tag name 'project' in totals output, got: {stdout}"
  );
}

#[test]
fn it_does_not_show_totals_without_flag() {
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
      "Yesterday no totals @project",
    ])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  let with_totals = doing.run(["yesterday", "--totals"]).output().expect("failed to run");
  let without_totals = doing.run(["yesterday"]).output().expect("failed to run");

  let with_stdout = String::from_utf8_lossy(&with_totals.stdout);
  let without_stdout = String::from_utf8_lossy(&without_totals.stdout);

  assert!(
    with_stdout.len() > without_stdout.len(),
    "expected --totals output to be longer than default output.\nWith totals ({} bytes): {with_stdout}\nWithout totals ({} bytes): {without_stdout}",
    with_stdout.len(),
    without_stdout.len()
  );
}
