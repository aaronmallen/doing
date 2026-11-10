use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_search() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Specific task\n\t- 2026-03-22 10:00 | Other task\n",
  )
  .expect("failed to write doing file");

  doing.run(["reset", "--search", "Specific task"]).assert().success();

  let contents = doing.read_doing_file();
  let specific_line = contents
    .lines()
    .find(|l| l.contains("Specific task"))
    .expect("expected Specific task");
  assert!(
    !specific_line.contains("2026-03-22 15:00"),
    "expected Specific task time to be reset, got: {specific_line}"
  );

  let other_line = contents
    .lines()
    .find(|l| l.contains("Other task"))
    .expect("expected Other task");
  assert!(
    other_line.contains("2026-03-22 10:00"),
    "expected Other task to keep original time, got: {other_line}"
  );
}
