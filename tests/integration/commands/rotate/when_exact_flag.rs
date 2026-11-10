use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_exact_matching() {
  let doing = DoingCmd::new();

  doing.run(["now", "Exact match task"]).assert().success();
  doing.run(["now", "Partial match"]).assert().success();

  doing
    .run(["rotate", "--search", "Exact match task", "--exact"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Exact match task"),
    "expected exactly matched entry to be rotated, got: {contents}"
  );
  assert!(
    contents.contains("Partial match"),
    "expected non-matching entry to remain, got: {contents}"
  );
}

#[test]
fn it_uses_exact_matching_with_short_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Exact match task"]).assert().success();
  doing.run(["now", "Partial match"]).assert().success();

  doing
    .run(["rotate", "--search", "Exact match task", "-x"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Exact match task"),
    "expected exactly matched entry to be rotated with -x flag, got: {contents}"
  );
  assert!(
    contents.contains("Partial match"),
    "expected non-matching entry to remain with -x flag, got: {contents}"
  );
}
