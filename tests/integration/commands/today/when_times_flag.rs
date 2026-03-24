use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_times() {
  let doing = DoingCmd::new();

  // Use absolute times firmly within today to avoid midnight timezone flakes on CI.
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", &format!("{today} 10:00"), "Times test"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 11:00")])
    .assert()
    .success();

  let output = doing.run(["today", "--times"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Times test"), "expected entry in output, got: {stdout}");
}

#[test]
fn it_hides_times() {
  let doing = DoingCmd::new();

  // Use absolute times firmly within today to avoid midnight timezone flakes on CI.
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", &format!("{today} 10:00"), "No times test"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 11:00")])
    .assert()
    .success();

  let output = doing.run(["today", "--no-times"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("No times test"),
    "expected entry in output, got: {stdout}"
  );
}

#[test]
fn it_uses_short_flag() {
  let doing = DoingCmd::new();

  // Use absolute times firmly within today to avoid midnight timezone flakes on CI.
  let today = chrono::Local::now().format("%Y-%m-%d").to_string();

  doing
    .run(["now", "--from", &format!("{today} 10:00"), "Short times test"])
    .assert()
    .success();
  doing
    .run(["done", "--at", &format!("{today} 11:00")])
    .assert()
    .success();

  let output = doing.run(["today", "-t"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Short times test"),
    "expected entry in output, got: {stdout}"
  );
}
