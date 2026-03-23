use crate::support::helpers::{DoingCmd, assert_times_within_tolerance, extract_entry_timestamp, fmt_time};

#[test]
fn it_adds_entry_to_currently_section_with_current_timestamp() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());

  doing.run(["now", "Test entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Currently"),
    "expected Currently section header, got: {contents}"
  );
  assert!(contents.contains("Test entry"), "expected entry text, got: {contents}");

  let entry_time = extract_entry_timestamp(&contents);
  assert_times_within_tolerance(&entry_time, &now, 1, "entry timestamp should be close to now");
}

#[test]
fn it_converts_multiple_parentheticals_as_single_note() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "Multiple (first paren) and (second paren)"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Multiple"),
    "expected title to contain text before first paren, got: {contents}"
  );
  assert!(
    contents.contains("\t\tfirst paren) and (second paren"),
    "expected everything from first ( to last ) as note, got: {contents}"
  );
}

#[test]
fn it_converts_trailing_parenthetical_to_note() {
  let doing = DoingCmd::new();

  doing.run(["now", "Adding entry (with a note)"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Adding entry") && !contents.contains("Adding entry (with a note)"),
    "expected parenthetical removed from title, got: {contents}"
  );
  assert!(
    contents.contains("\t\twith a note"),
    "expected parenthetical converted to indented note, got: {contents}"
  );
}

#[test]
fn it_outputs_status_to_stderr() {
  let doing = DoingCmd::new();

  let output = doing.run(["now", "Status test"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(stdout.is_empty(), "expected stdout to be empty, got: {stdout}");
  assert!(
    stderr.contains("New entry:"),
    "expected status message on stderr, got: {stderr}"
  );
}

#[test]
fn it_preserves_inline_tags() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @project2 @urgent"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Task @project2 @urgent"),
    "expected inline tags to be preserved, got: {contents}"
  );
}

#[test]
fn it_treats_inline_done_tag_as_regular_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @done"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Task @done"),
    "expected @done preserved as-is (no timestamp added), got: {contents}"
  );
  assert!(
    !contents.contains("@done("),
    "expected @done without timestamp, got: {contents}"
  );
}
