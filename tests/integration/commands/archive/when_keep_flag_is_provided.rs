use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_keeps_n_entries_in_source_section() {
  let doing = DoingCmd::new();

  // Use entries with distinct timestamps so ordering is deterministic
  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-01 10:00 | Task one <aaa001>
\t- 2024-01-02 10:00 | Task two <aaa002>
\t- 2024-01-03 10:00 | Task three <aaa003>
",
  )
  .expect("failed to write doing file");

  doing.run(["archive", "--keep", "1"]).assert().success();

  let contents = doing.read_doing_file();

  // Most recent entry should remain in Currently
  let currently_section = contents.split("Archive:").next().unwrap_or("");
  assert!(
    currently_section.contains("Task three"),
    "expected most recent entry to remain in Currently, got: {contents}"
  );

  // Older entries should be in Archive
  assert!(
    contents.contains("Archive:"),
    "expected Archive section to exist, got: {contents}"
  );
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task one") && archive_section.contains("Task two"),
    "expected older entries in Archive, got: {contents}"
  );
}

#[test]
fn it_archives_nothing_when_keep_exceeds_entry_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "Only task"]).assert().success();

  let output = doing.run(["archive", "--keep", "5"]).output().expect("failed to run");

  assert!(output.status.success());

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Only task"),
    "expected entry to remain when keep exceeds count, got: {contents}"
  );

  // Entry should not be in Archive
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    !archive_section.contains("Only task"),
    "expected no entries in Archive when keep exceeds count, got: {contents}"
  );
}

#[test]
fn it_is_ignored_when_filtering_by_tag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-01 10:00 | Task one @project1 <aaa001>
\t- 2024-01-02 10:00 | Task two @project1 <aaa002>
\t- 2024-01-03 10:00 | Task three <aaa003>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--tag", "project1", "--keep", "1"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // When filtering by tag, --keep should be ignored, so both tagged entries should be archived
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task one") && archive_section.contains("Task two"),
    "expected both tagged entries to be archived (--keep ignored with --tag), got: {contents}"
  );
}

#[test]
fn it_is_ignored_when_filtering_by_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-01 10:00 | Task Alpha <aaa001>
\t- 2024-01-02 10:00 | Task Beta <aaa002>
\t- 2024-01-03 10:00 | Task Alpha again <aaa003>
",
  )
  .expect("failed to write doing file");

  doing
    .run(["archive", "--search", "Alpha", "--keep", "1"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // When filtering by search, --keep should be ignored, so both matching entries should be archived
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task Alpha") && archive_section.contains("Task Alpha again"),
    "expected both matching entries to be archived (--keep ignored with --search), got: {contents}"
  );
}
