use pretty_assertions::assert_eq;

use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "tags --no-counts flag not supported (see #208)"]
fn it_hides_counts_when_disabled() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one @project"]).assert().success();
  doing.run(["now", "Task two @project"]).assert().success();

  let output = doing.run(["tags", "--no-counts"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines, vec!["project"]);
}

#[test]
fn it_shows_counts_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one @project"]).assert().success();
  doing.run(["now", "Task two @project"]).assert().success();

  let output = doing.run(["tags", "-c"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("project") && stdout.contains("2"),
    "expected project with count 2 using -c, got: {stdout}"
  );
}

#[test]
fn it_shows_occurrence_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one @project @coding"]).assert().success();
  doing.run(["now", "Task two @project"]).assert().success();
  doing.run(["now", "Task three @project @review"]).assert().success();

  let output = doing.run(["tags", "--counts"]).output().expect("failed to run");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(
    lines.iter().any(|l| l.contains("project") && l.contains("3")),
    "expected project with count 3, got: {stdout}"
  );
  assert!(
    lines.iter().any(|l| l.contains("coding") && l.contains("1")),
    "expected coding with count 1, got: {stdout}"
  );
  assert!(
    lines.iter().any(|l| l.contains("review") && l.contains("1")),
    "expected review with count 1, got: {stdout}"
  );
}
