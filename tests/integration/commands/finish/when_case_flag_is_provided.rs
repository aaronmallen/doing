use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_respects_case_sensitivity_in_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 11:00 | Task UPPER\n\t- 2026-03-22 10:00 | Task upper\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["finish", "--search", "UPPER", "--case", "c", "5"])
    .assert()
    .success();

  let contents = doing.read_doing_file();

  // Case-sensitive: only "UPPER" should match
  let upper_line = contents.lines().find(|l| l.contains("UPPER")).unwrap();
  assert!(
    upper_line.contains("@done("),
    "expected UPPER entry to be finished, got: {upper_line}"
  );

  let lower_line = contents.lines().find(|l| l.contains("Task upper")).unwrap();
  assert!(
    !lower_line.contains("@done"),
    "expected lowercase 'upper' entry to remain unfinished, got: {lower_line}"
  );
}
