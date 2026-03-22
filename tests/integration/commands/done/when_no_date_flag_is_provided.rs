use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_done_without_timestamp() {
  let doing = DoingCmd::new();

  doing.run(["done", "--no-date", "No date entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected @done tag present, got: {contents}"
  );
  assert!(
    !contents.contains("@done("),
    "expected @done without parenthesized date, got: {contents}"
  );
}
