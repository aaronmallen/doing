use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "rotate ignores --search/--case filter, rotates all entries (see #184)"]
fn it_performs_case_sensitive_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "UPPERCASE task"]).assert().success();
  doing.run(["now", "lowercase task"]).assert().success();

  doing
    .run(["rotate", "--search", "UPPERCASE", "--case", "sensitive"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("UPPERCASE task"),
    "expected case-matched entry to be rotated, got: {contents}"
  );
  assert!(
    contents.contains("lowercase task"),
    "expected non-matching entry to remain, got: {contents}"
  );
}
