use pretty_assertions::assert_eq;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "tags outputs @-prefixed names instead of bare names (see #207)"]
fn it_sorts_by_name() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @zebra"]).assert().success();
  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @middle"]).assert().success();

  let output = doing.run(["tags", "--sort", "name"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines, vec!["alpha", "middle", "zebra"]);
}

#[test]
#[ignore = "tags outputs @-prefixed names instead of bare names (see #207)"]
fn it_sorts_by_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @rare"]).assert().success();
  doing.run(["now", "Task @common @rare"]).assert().success();
  doing.run(["now", "Task @common"]).assert().success();
  doing.run(["now", "Task @common"]).assert().success();

  let output = doing.run(["tags", "--sort", "count"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  // sort by count ascending: rare (2) then common (3)
  assert_eq!(lines, vec!["rare", "common"]);
}
