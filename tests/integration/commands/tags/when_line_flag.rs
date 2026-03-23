use pretty_assertions::assert_eq;

use crate::support::helpers::DoingCmd;

#[test]
fn it_disables_single_line() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @beta"]).assert().success();

  let output = doing.run(["tags", "--no-line"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines, vec!["alpha", "beta"]);
}

#[test]
fn it_outputs_single_line() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @beta"]).assert().success();
  doing.run(["now", "Task @gamma"]).assert().success();

  let output = doing.run(["tags", "--line"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let trimmed = stdout.trim();

  assert_eq!(trimmed, "@alpha @beta @gamma");
}

#[test]
fn it_outputs_single_line_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @beta"]).assert().success();

  let output = doing.run(["tags", "-l"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let trimmed = stdout.trim();

  assert_eq!(trimmed, "@alpha @beta");
}
