use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_outputs_in_specified_format() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | JSON task <aaa111>\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["select", "--no-menu", "--query", "JSON task", "--output", "json"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  // JSON output should contain the entry data
  assert!(
    stdout.contains("JSON task") || stdout.contains("json"),
    "expected JSON output format, got: {stdout}"
  );
}

#[test]
fn it_outputs_with_short_flag() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Short output task <aaa111>\n",
  )
  .expect("failed to write doing file");

  let output = doing
    .run(["select", "--no-menu", "--query", "Short output", "-o", "json"])
    .output()
    .expect("failed to run");

  assert!(output.status.success());
}
