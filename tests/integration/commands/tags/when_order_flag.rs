use pretty_assertions::assert_eq;

use crate::support::helpers::DoingCmd;

#[test]
fn it_orders_ascending() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @zebra"]).assert().success();
  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @middle"]).assert().success();

  let output = doing.run(["tags", "--order", "asc"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines, vec!["alpha", "middle", "zebra"]);
}

#[test]
fn it_orders_ascending_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @zebra"]).assert().success();
  doing.run(["now", "Task @alpha"]).assert().success();

  let output = doing.run(["tags", "-o", "asc"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines, vec!["alpha", "zebra"]);
}

#[test]
fn it_orders_descending() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @middle"]).assert().success();
  doing.run(["now", "Task @zebra"]).assert().success();

  let output = doing.run(["tags", "--order", "desc"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines, vec!["zebra", "middle", "alpha"]);
}
