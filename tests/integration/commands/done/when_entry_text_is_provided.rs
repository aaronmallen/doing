use crate::support::helpers::{DoingCmd, extract_done_timestamp, extract_entry_timestamp, fmt_time};

#[test]
fn it_adds_new_entry_immediately_marked_done() {
  let doing = DoingCmd::new();

  doing.run(["now", "Existing entry"]).assert().success();
  doing.run(["done", "New done entry"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("New done entry @done("),
    "expected new entry to have @done tag, got: {contents}"
  );

  // Existing entry should NOT be marked done
  let existing_line = contents
    .lines()
    .find(|l| l.contains("Existing entry"))
    .expect("expected Existing entry in doing file");
  assert!(
    !existing_line.contains("@done"),
    "expected existing entry to remain unchanged, got: {existing_line}"
  );
}

#[test]
fn it_preserves_inline_tags() {
  let doing = DoingCmd::new();

  doing.run(["done", "Tagged entry @project1 @urgent"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@project1"),
    "expected @project1 tag preserved, got: {contents}"
  );
  assert!(
    contents.contains("@urgent"),
    "expected @urgent tag preserved, got: {contents}"
  );
  assert!(contents.contains("@done("), "expected @done tag added, got: {contents}");
}

#[test]
fn it_sets_start_and_done_to_current_time() {
  let doing = DoingCmd::new();
  let now = fmt_time(chrono::Local::now());

  doing.run(["done", "Zero elapsed entry"]).assert().success();

  let contents = doing.read_doing_file();
  let start_time = extract_entry_timestamp(&contents);
  let done_time = extract_done_timestamp(&contents);

  crate::support::helpers::assert_times_within_tolerance(&start_time, &now, 1, "start time should be close to now");
  crate::support::helpers::assert_times_within_tolerance(&done_time, &now, 1, "done time should be close to now");
}
