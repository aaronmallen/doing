use crate::support::helpers::DoingCmd;

#[test]
fn it_removes_done_tag_from_last_entry() {
  let doing = DoingCmd::new();

  // Add an entry and mark it done
  doing.run(["now", "Entry to undone"]).assert().success();
  doing.run(["done"]).assert().success();

  // Verify it's marked done
  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done("),
    "expected @done tag before removal, got: {contents}"
  );

  // Remove the @done tag
  doing.run(["done", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Entry to undone"),
    "expected entry to still exist, got: {contents}"
  );
  assert!(
    !contents.contains("@done"),
    "expected @done tag to be removed, got: {contents}"
  );
}
