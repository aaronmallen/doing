use crate::support::helpers::{DoingCmd, count_entries};

#[test]
fn it_shows_specified_number_of_entries() {
  let doing = DoingCmd::new();

  for i in 1..=10 {
    doing.run(["now", &format!("Entry {i}")]).assert().success();
  }

  let output = doing.run(["recent", "5"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_count = count_entries(&stdout);
  assert_eq!(
    entry_count, 5,
    "expected exactly 5 entries, got {entry_count} in: {stdout}"
  );
}

#[test]
fn it_shows_all_when_count_exceeds_entries() {
  let doing = DoingCmd::new();

  for i in 1..=3 {
    doing.run(["now", &format!("Entry {i}")]).assert().success();
  }

  let output = doing.run(["recent", "100"]).output().expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let entry_count = count_entries(&stdout);
  assert_eq!(
    entry_count, 3,
    "expected all 3 entries when count exceeds total, got {entry_count} in: {stdout}"
  );
}
