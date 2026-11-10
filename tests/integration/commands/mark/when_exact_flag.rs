use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_exact_matching() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Important Task\n\t- 2026-03-22 14:00 | Another task\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["mark", "--search", "Important Task", "--exact"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let important_line = contents
    .lines()
    .find(|l| l.contains("Important Task"))
    .expect("expected Important Task");
  assert!(
    important_line.contains("@flagged"),
    "expected exact match to flag entry, got: {important_line}"
  );
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2026-03-22 15:00 | Important Task\n\t- 2026-03-22 14:00 | Another task\n",
  )
  .expect("failed to write doing file");

  doing
    .run(["mark", "--search", "Important Task", "-x"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let important_line = contents
    .lines()
    .find(|l| l.contains("Important Task"))
    .expect("expected Important Task");
  assert!(
    important_line.contains("@flagged"),
    "expected exact match with -x to flag entry, got: {important_line}"
  );
}
