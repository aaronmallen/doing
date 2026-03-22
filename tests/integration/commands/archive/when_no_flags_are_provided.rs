use crate::support::helpers::DoingCmd;

#[test]
fn it_moves_all_entries_from_currently_to_archive() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one"]).assert().success();
  doing.run(["now", "Task two"]).assert().success();

  doing.run(["archive"]).assert().success();

  let contents = doing.read_doing_file();

  // Currently section should be empty of entries
  let currently_section = contents.split("Archive:").next().unwrap_or("");
  assert!(
    !currently_section.contains("Task one") && !currently_section.contains("Task two"),
    "expected Currently section to be empty after archive, got: {contents}"
  );

  // Archive section should contain the entries
  assert!(
    contents.contains("Task one") && contents.contains("Task two"),
    "expected entries in Archive section, got: {contents}"
  );
}

#[test]
fn it_adds_from_label_to_each_moved_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Labeled task"]).assert().success();
  doing.run(["archive"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@from(Currently)"),
    "expected @from(Currently) label on archived entry, got: {contents}"
  );
}

#[test]
fn it_outputs_archived_message_to_stderr() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task for stderr"]).assert().success();

  let output = doing.run(["archive"]).output().expect("failed to run");

  assert!(output.status.success());

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("Archived"),
    "expected 'Archived' message on stderr, got: {stderr}"
  );
}

#[test]
fn it_skips_when_section_is_empty() {
  let doing = DoingCmd::new();

  let output = doing.run(["archive"]).output().expect("failed to run");

  assert!(output.status.success(), "expected exit 0 when section is empty");

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stderr.contains("No items") || stderr.contains("No entries") || stderr.is_empty(),
    "expected skip message or empty stderr when nothing to archive, got: {stderr}"
  );
}
