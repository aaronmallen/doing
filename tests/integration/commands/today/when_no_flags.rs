use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_entries_from_today() {
  let doing = DoingCmd::new();

  doing.run(["now", "Today entry"]).assert().success();
  doing.run(["now", "Another today entry"]).assert().success();

  let output = doing.run(["today"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Today entry"),
    "expected 'Today entry' in output, got: {stdout}"
  );
  assert!(
    stdout.contains("Another today entry"),
    "expected 'Another today entry' in output, got: {stdout}"
  );
}

#[test]
fn it_shows_nothing_when_no_entries_today() {
  let doing = DoingCmd::new();

  let output = doing.run(["today"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.trim().is_empty(),
    "expected empty output when no entries today, got: {stdout}"
  );
}

#[test]
fn it_shows_entries_from_all_sections() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry in Currently"]).assert().success();
  doing
    .run(["now", "--section", "Later", "Entry in Later"])
    .assert()
    .success();

  let output = doing.run(["today"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Entry in Currently"),
    "expected entry from Currently section, got: {stdout}"
  );
  assert!(
    stdout.contains("Entry in Later"),
    "expected entry from Later section, got: {stdout}"
  );
}
