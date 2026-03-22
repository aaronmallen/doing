use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_done_timestamp, fmt_time};

#[test]
fn it_does_not_update_already_done_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "First @done(2026-01-01 12:00)"]).assert().success();
  doing.run(["now", "--finish-last", "Second entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done(2026-01-01 12:00)"),
    "expected original @done timestamp preserved, got: {contents}"
  );
}

#[test]
fn it_errors_when_section_is_empty() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["now", "--finish-last", "Entry in empty section"])
    .output()
    .expect("failed to run");

  let stderr = String::from_utf8_lossy(&output.stderr);
  let stdout = String::from_utf8_lossy(&output.stdout);

  if !output.status.success() {
    assert!(
      !stderr.is_empty(),
      "expected error message on stderr when section is empty, stdout: {stdout}"
    );
  }
  if output.status.success() {
    let contents = doing.read_doing_file();
    assert!(
      contents.contains("Entry in empty section"),
      "expected entry to be created even with empty section, got: {contents}"
    );
  }
}

#[test]
fn it_marks_previous_entry_as_done() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "--finish-last", "Second entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done("),
    "expected previous entry to have @done tag, got: {contents}"
  );
  assert!(
    contents.contains("First entry") && contents.contains("Second entry"),
    "expected both entries, got: {contents}"
  );
}

#[test]
fn it_sets_done_time_to_match_new_entry_start() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "--finish-last", "Second entry"]).assert().success();

  let contents = doing.read_doing_file();
  let done_time = extract_done_timestamp(&contents);
  assert_times_within_tolerance(&done_time, &now, 1, "@done timestamp should match new entry start");
}
