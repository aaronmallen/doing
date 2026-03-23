use crate::support::helpers::{DoingCmd, extract_done_timestamp, fmt_time};

#[test]
fn it_marks_last_entry_as_done_with_current_timestamp() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();
  doing.run(["done"]).assert().success();

  let contents = doing.read_doing_file();

  // Entry two (most recent) should have @done
  assert!(
    contents.contains("Entry two @done("),
    "expected most recent entry to be marked @done, got: {contents}"
  );

  // Entry one should NOT have @done
  let entry_one_line = contents
    .lines()
    .find(|l| l.contains("Entry one"))
    .expect("expected Entry one in doing file");
  assert!(
    !entry_one_line.contains("@done"),
    "expected Entry one to remain unchanged, got: {entry_one_line}"
  );

  let done_time = extract_done_timestamp(&contents);
  crate::support::helpers::assert_times_within_tolerance(&done_time, &now, 1, "@done timestamp should be close to now");
}

#[test]
fn it_exits_with_error_when_section_is_empty() {
  let doing = DoingCmd::new();

  let output = doing.run(["done"]).output().expect("failed to run");

  assert!(!output.status.success(), "expected non-zero exit code");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("no items matched"),
    "expected 'no items matched' error on stderr, got: {stderr}"
  );
}

#[test]
#[ignore = "status message format differs from Ruby doing (see #159)"]
fn it_outputs_status_to_stderr() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry for done"]).assert().success();

  let output = doing.run(["done"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(stdout.is_empty(), "expected stdout to be empty, got: {stdout}");
  assert!(
    stderr.contains("Tagged:"),
    "expected 'Tagged:' status message on stderr, got: {stderr}"
  );
}
