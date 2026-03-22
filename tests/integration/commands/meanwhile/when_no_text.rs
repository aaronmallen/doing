use crate::support::helpers::DoingCmd;

#[test]
fn it_finishes_current_meanwhile() {
  let doing = DoingCmd::new();

  doing.run(["meanwhile", "Active meanwhile"]).assert().success();
  doing.run(["meanwhile"]).assert().success();

  let contents = doing.read_doing_file();

  let entry_line = contents
    .lines()
    .find(|l| l.contains("Active meanwhile"))
    .expect("expected Active meanwhile entry");
  assert!(
    entry_line.contains("@done("),
    "expected @done on meanwhile entry after finishing, got: {entry_line}"
  );
}

#[test]
fn it_does_nothing_when_no_meanwhile_exists() {
  let doing = DoingCmd::new();

  // Add a regular entry (no @meanwhile)
  doing.run(["now", "Regular entry"]).assert().success();

  // Running meanwhile with no text and no existing @meanwhile
  doing.run(["meanwhile"]).assert().success();

  let contents_after = doing.read_doing_file();

  // Regular entry should remain unchanged
  let entry_line = contents_after
    .lines()
    .find(|l| l.contains("Regular entry"))
    .expect("expected Regular entry to still exist");
  assert!(
    !entry_line.contains("@done"),
    "expected regular entry to remain unchanged, got: {entry_line}"
  );

  // No new entries should be added
  assert!(
    !contents_after.contains("@meanwhile"),
    "expected no @meanwhile entries, got: {contents_after}"
  );
}
