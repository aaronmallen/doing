use crate::support::helpers::DoingCmd;

#[test]
fn it_finishes_last_entry_not_already_done() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry one"]).assert().success();
  doing.run(["now", "Entry two"]).assert().success();

  // Mark entry two (most recent) as done
  doing.run(["done"]).assert().success();

  // Verify entry two is done, entry one is not
  let contents = doing.read_doing_file();
  let entry_two_line = contents.lines().find(|l| l.contains("Entry two")).unwrap();
  assert!(entry_two_line.contains("@done"), "entry two should be done");
  let entry_one_line = contents.lines().find(|l| l.contains("Entry one")).unwrap();
  assert!(!entry_one_line.contains("@done"), "entry one should not be done yet");

  // Use --unfinished to mark the first unfinished entry as done
  doing.run(["done", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();
  let entry_one_line = contents.lines().find(|l| l.contains("Entry one")).unwrap();
  assert!(
    entry_one_line.contains("@done"),
    "expected --unfinished to mark Entry one as done, got: {entry_one_line}"
  );
}
