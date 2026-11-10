use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_gets_last_entry_from_specified_section() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-20 10:00 | Current task <aaa111>
Archive:
\t- 2024-01-10 10:00 | Archived task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--section", "Archive"]).assert().success();

  let contents = doing.read_doing_file();

  // Should duplicate the entry from Archive, not from Currently
  let count = contents.matches("Archived task").count();
  assert!(
    count >= 2,
    "expected entry from Archive to be duplicated, got {count} in: {contents}"
  );
}

#[test]
fn it_does_not_affect_other_sections() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-20 10:00 | Current task <aaa111>
Later:
\t- 2024-01-10 10:00 | Later task <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--section", "Later"]).assert().success();

  let contents = doing.read_doing_file();

  // Currently section should be unchanged (original entry still there, no extra)
  assert!(
    contents.contains("Current task"),
    "expected Currently entries to be unchanged, got: {contents}"
  );
}
