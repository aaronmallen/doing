use crate::support::helpers::DoingCmd;

#[test]
fn it_limits_rotation_to_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Current task"]).assert().success();
  doing.run(["now", "-s", "Later", "Later task"]).assert().success();

  doing.run(["rotate", "--section", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Current task"),
    "expected Currently entries to remain when rotating a different section, got: {contents}"
  );
  assert!(
    !contents.contains("Later task"),
    "expected Later entries to be rotated, got: {contents}"
  );
}

#[test]
fn it_limits_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Current task"]).assert().success();
  doing.run(["now", "-s", "Later", "Later task"]).assert().success();

  doing.run(["rotate", "-s", "Later"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Current task"),
    "expected Currently entries to remain with -s flag, got: {contents}"
  );
  assert!(
    !contents.contains("Later task"),
    "expected Later entries to be rotated with -s flag, got: {contents}"
  );
}
