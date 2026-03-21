use crate::helpers::DoingCmd;

#[test]
fn it_adds_date_stamped_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test date tag"]).assert().success();
  doing.run(["tag", "--date", "dated"]).assert().success();

  let contents = doing.read_doing_file();
  let re = regex::Regex::new(r"@dated\(\d{4}-\d{2}-\d{2} \d{2}:\d{2}\)").unwrap();
  assert!(
    re.is_match(&contents),
    "entry should have @dated with a date and time value"
  );
}

#[test]
fn it_adds_tag_to_last_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test new entry"]).assert().success();
  doing.run(["tag", "testtag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@testtag"), "entry should have @testtag");
}

#[test]
fn it_adds_tag_with_value() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test value tag"]).assert().success();
  doing.run(["tag", "--value", "myval", "testtag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@testtag(myval)"),
    "entry should have @testtag(myval)"
  );
}

#[test]
fn it_removes_tag_from_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry @removeme"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@removeme"), "tag should be present before removal");

  doing.run(["tag", "--remove", "removeme"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(!contents.contains("@removeme"), "tag should be removed from entry");
}

#[test]
fn it_tags_entries_matching_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Alpha target entry"]).assert().success();
  doing.run(["now", "Beta other entry"]).assert().success();
  doing.run(["now", "Gamma target entry"]).assert().success();

  doing
    .run(["tag", "--search", "target", "--force", "found"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let lines: Vec<&str> = contents.lines().collect();

  let alpha = lines
    .iter()
    .find(|l| l.contains("Alpha"))
    .expect("should have Alpha entry");
  let beta = lines
    .iter()
    .find(|l| l.contains("Beta"))
    .expect("should have Beta entry");
  let gamma = lines
    .iter()
    .find(|l| l.contains("Gamma"))
    .expect("should have Gamma entry");

  assert!(alpha.contains("@found"), "Alpha should be tagged @found");
  assert!(!beta.contains("@found"), "Beta should not be tagged @found");
  assert!(gamma.contains("@found"), "Gamma should be tagged @found");
}

#[test]
fn it_tags_last_unfinished_entry_with_unfinished_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Active task"]).assert().success();
  doing.run(["now", "Finished task"]).assert().success();
  doing.run(["done"]).assert().success();

  // Tag with --unfinished should apply to "Active task", skipping the done entry
  doing.run(["tag", "--unfinished", "important"]).assert().success();

  let content = doing.read_doing_file();
  let active_line = content
    .lines()
    .find(|l| l.contains("Active task"))
    .expect("should have Active task");
  assert!(
    active_line.contains("@important"),
    "expected @important tag on Active task"
  );
}

#[test]
fn it_tags_last_n_entries_with_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry"]).assert().success();
  doing.run(["now", "Third entry"]).assert().success();

  doing
    .run(["tag", "--count", "2", "--force", "counted"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  let lines: Vec<&str> = contents.lines().collect();

  let first = lines
    .iter()
    .find(|l| l.contains("First"))
    .expect("should have First entry");
  let second = lines
    .iter()
    .find(|l| l.contains("Second"))
    .expect("should have Second entry");
  let third = lines
    .iter()
    .find(|l| l.contains("Third"))
    .expect("should have Third entry");

  assert!(!first.contains("@counted"), "First entry should not be tagged");
  assert!(second.contains("@counted"), "Second entry should be tagged");
  assert!(third.contains("@counted"), "Third entry should be tagged");
}

#[test]
fn it_updates_existing_tag_value() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test update tag"]).assert().success();
  doing.run(["tag", "--value", "old", "status"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@status(old)"), "should have @status(old)");

  doing.run(["tag", "--value", "new", "status"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@status(new)"), "should have updated to @status(new)");
  assert!(!contents.contains("@status(old)"), "old value should be gone");
}
