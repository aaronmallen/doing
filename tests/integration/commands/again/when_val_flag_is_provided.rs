use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_value_comparison() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Low progress @progress(30) <aaa111>
\t- 2024-01-11 10:00 | High progress @progress(80) <bbb222>
",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--val", "@progress > 60"]).assert().success();

  let contents = doing.read_doing_file();

  let count = contents.matches("High progress").count();
  assert!(
    count >= 2,
    "expected high-progress entry to be duplicated, got {count} in: {contents}"
  );
}

#[test]
fn it_skips_when_no_entries_match_value_query() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | Low progress @progress(30) <aaa111>
",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["again", "--val", "@progress > 90"])
    .output()
    .expect("failed to run");

  assert!(
    output.status.success(),
    "expected exit 0 when no entries match val query"
  );
}
