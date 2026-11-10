use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_default_number_of_recent_entries() {
  let doing = DoingCmd::new();

  for i in 1..=5 {
    doing.run(["now", &format!("Entry {i}")]).assert().success();
  }

  let output = doing.run(["recent"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Entry 5"),
    "expected most recent entry in output, got: {stdout}"
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

  let output = doing.run(["recent"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Entry in Currently"),
    "expected entry from Currently, got: {stdout}"
  );
  assert!(
    stdout.contains("Entry in Later"),
    "expected entry from Later, got: {stdout}"
  );
}
