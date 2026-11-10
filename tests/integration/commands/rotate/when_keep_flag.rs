use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_retains_n_most_recent_per_section() {
  let doing = DoingCmd::new();

  // Use entries with distinct timestamps so "most recent" is deterministic
  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-01 10:00 | Task one <aaa001>
\t- 2024-01-02 10:00 | Task two <aaa002>
\t- 2024-01-03 10:00 | Task three <aaa003>
\t- 2024-01-04 10:00 | Task four <aaa004>
\t- 2024-01-05 10:00 | Task five <aaa005>
",
  )
  .expect("failed to write doing file");

  doing.run(["rotate", "--keep", "3"]).assert().success();

  let contents = doing.read_doing_file();

  // The 3 most recent entries should remain
  assert!(
    contents.contains("Task five"),
    "expected most recent entry to be kept, got: {contents}"
  );
  assert!(
    contents.contains("Task four"),
    "expected second most recent entry to be kept, got: {contents}"
  );
  assert!(
    contents.contains("Task three"),
    "expected third most recent entry to be kept, got: {contents}"
  );

  // The older entries should be rotated out
  assert!(
    !contents.contains("Task one"),
    "expected older entry to be rotated out, got: {contents}"
  );
  assert!(
    !contents.contains("Task two"),
    "expected older entry to be rotated out, got: {contents}"
  );
}

#[test]
fn it_retains_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-01 10:00 | Task one <bbb001>
\t- 2024-01-02 10:00 | Task two <bbb002>
\t- 2024-01-03 10:00 | Task three <bbb003>
",
  )
  .expect("failed to write doing file");

  doing.run(["rotate", "-k", "1"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Task three"),
    "expected most recent entry to be kept with -k flag, got: {contents}"
  );
  assert!(
    !contents.contains("Task one"),
    "expected older entry to be rotated with -k flag, got: {contents}"
  );
}

#[test]
fn it_keeps_entries_per_section_independently() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-01 10:00 | Current one <ccc001>
\t- 2024-01-02 10:00 | Current two <ccc002>
\t- 2024-01-03 10:00 | Current three <ccc003>
\t- 2024-01-04 10:00 | Current four <ccc004>
Later:
\t- 2024-01-01 10:00 | Later one <ddd001>
\t- 2024-01-02 10:00 | Later two <ddd002>
\t- 2024-01-03 10:00 | Later three <ddd003>
\t- 2024-01-04 10:00 | Later four <ddd004>
",
  )
  .expect("failed to write doing file");

  doing.run(["rotate", "--keep", "2"]).assert().success();

  let contents = doing.read_doing_file();

  // Should keep 2 per section
  assert!(
    contents.contains("Current four"),
    "expected most recent Currently entry to be kept, got: {contents}"
  );
  assert!(
    contents.contains("Current three"),
    "expected second most recent Currently entry to be kept, got: {contents}"
  );
  assert!(
    !contents.contains("Current one"),
    "expected older Currently entry to be rotated, got: {contents}"
  );

  assert!(
    contents.contains("Later four"),
    "expected most recent Later entry to be kept, got: {contents}"
  );
  assert!(
    contents.contains("Later three"),
    "expected second most recent Later entry to be kept, got: {contents}"
  );
  assert!(
    !contents.contains("Later one"),
    "expected older Later entry to be rotated, got: {contents}"
  );
}
