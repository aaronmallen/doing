use crate::helpers::DoingCmd;

#[test]
fn it_finishes_entries_matching_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "Searchable task"]).assert().success();
  doing.run(["now", "Other task"]).assert().success();

  doing
    .run(["select", "--no-menu", "--query", "Searchable", "--finish"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(content.contains("Searchable task"), "entry should still exist");
  assert!(content.contains("@done"), "matched entry should be marked @done");
}

#[test]
fn it_cancels_entries_matching_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "Cancel this task"]).assert().success();
  doing.run(["now", "Keep this task"]).assert().success();

  doing
    .run(["select", "--no-menu", "--query", "Cancel this", "--cancel"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(content.contains("@done"), "cancelled entry should have @done tag");
}

#[test]
fn it_archives_entries_matching_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "Archive this task"]).assert().success();
  doing.run(["now", "Keep this task"]).assert().success();

  doing
    .run(["select", "--no-menu", "--query", "Archive this", "--archive"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(content.contains("Archive:"), "Archive section should be created");
  assert!(
    content.contains("Archive this task"),
    "archived entry should be in Archive section"
  );
}

#[test]
fn it_deletes_entries_matching_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "Delete this task"]).assert().success();
  doing.run(["now", "Keep this task"]).assert().success();

  doing
    .run(["select", "--no-menu", "--query", "Delete this", "--delete"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(
    !content.contains("Delete this task"),
    "deleted entry should not appear in doing file"
  );
  assert!(content.contains("Keep this task"), "non-matching entry should remain");
}

#[test]
fn it_tags_entries_matching_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "Tag this task"]).assert().success();
  doing.run(["now", "Leave this task"]).assert().success();

  doing
    .run(["select", "--no-menu", "--query", "Tag this", "--tag", "newtag"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(content.contains("@newtag"), "matched entry should have the new tag");
}

#[test]
fn it_moves_entries_matching_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "Move this task"]).assert().success();
  doing.run(["now", "Stay here task"]).assert().success();

  doing
    .run(["select", "--no-menu", "--query", "Move this", "--move", "Later"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(content.contains("Later:"), "Later section should be created");
  assert!(content.contains("Move this task"), "moved entry should be in the file");
}

#[test]
fn it_flags_entries_matching_query() {
  let doing = DoingCmd::new();

  doing.run(["now", "Flag this task"]).assert().success();
  doing.run(["now", "Other task"]).assert().success();

  doing
    .run(["select", "--no-menu", "--query", "Flag this", "--flag"])
    .assert()
    .success();

  let content = doing.read_doing_file();
  assert!(content.contains("@flagged"), "matched entry should be flagged");
}

#[test]
fn it_handles_no_matching_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Some task"]).assert().success();

  let output = doing
    .run(["select", "--no-menu", "--query", "nonexistent", "--delete"])
    .output()
    .expect("failed to run select");

  assert!(output.status.success(), "select with no matches should succeed");
  let content = doing.read_doing_file();
  assert!(content.contains("Some task"), "existing entry should remain");
}
