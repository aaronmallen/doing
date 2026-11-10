use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_times() {
  let doing = DoingCmd::new();

  // Use absolute times on yesterday to avoid midnight timezone flakes on CI.
  let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
    .format("%Y-%m-%d")
    .to_string();

  doing
    .run(["now", "--from", &format!("{yesterday} 10:00"), "Yesterday times"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  let output = doing.run(["yesterday", "--times"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday times"),
    "expected entry in output, got: {stdout}"
  );
}

#[test]
fn it_hides_times() {
  let doing = DoingCmd::new();

  // Use absolute times on yesterday to avoid midnight timezone flakes on CI.
  let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
    .format("%Y-%m-%d")
    .to_string();

  doing
    .run(["now", "--from", &format!("{yesterday} 10:00"), "Yesterday no times"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  let output = doing.run(["yesterday", "--no-times"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}

#[test]
fn it_uses_short_flag() {
  let doing = DoingCmd::new();

  // Use absolute times on yesterday to avoid midnight timezone flakes on CI.
  let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
    .format("%Y-%m-%d")
    .to_string();

  doing
    .run(["now", "--from", &format!("{yesterday} 10:00"), "Yesterday short times"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{yesterday} 11:00")])
    .assert()
    .success();

  let output = doing.run(["yesterday", "-t"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday short times"),
    "expected entry in output, got: {stdout}"
  );
}
