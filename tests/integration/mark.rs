use crate::helpers::DoingCmd;

#[test]
fn it_adds_flagged_tag_to_last_entry() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test mark entry"]).assert().success();
  doing.run(["mark"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@flagged"), "entry should have @flagged after mark");
}

#[test]
fn it_marks_only_unfinished_entries_with_unfinished_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Active entry"]).assert().success();
  doing.run(["done", "Finished entry"]).assert().success();

  // --unfinished restricts to entries without @done
  doing.run(["mark", "--unfinished"]).assert().success();

  let contents = doing.read_doing_file();
  let active_line = contents
    .lines()
    .find(|l| l.contains("Active entry"))
    .expect("should have active entry");
  assert!(active_line.contains("@flagged"), "active entry should have @flagged");

  let done_line = contents
    .lines()
    .find(|l| l.contains("Finished entry"))
    .expect("should have finished entry");
  assert!(
    !done_line.contains("@flagged"),
    "finished entry should not have @flagged"
  );
}

#[test]
fn it_removes_flagged_tag_on_second_mark() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test toggle entry"]).assert().success();
  doing.run(["mark"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@flagged"),
    "entry should have @flagged after first mark"
  );

  doing.run(["mark"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@flagged"),
    "entry should not have @flagged after second mark"
  );
}

#[test]
fn it_removes_flagged_tag_with_remove_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test remove entry"]).assert().success();
  doing.run(["mark"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("@flagged"), "entry should have @flagged");

  doing.run(["mark", "--remove"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@flagged"),
    "entry should not have @flagged after --remove"
  );
}

#[test]
fn it_uses_configured_marker_tag() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
marker_tag = "important"
paginate = false

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;
  let doing = DoingCmd::new_with_config(config);

  doing.run(["now", "Test custom marker"]).assert().success();
  doing.run(["mark"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@important"),
    "entry should have @important (configured marker_tag)"
  );
  assert!(
    !contents.contains("@flagged"),
    "entry should not have @flagged when marker_tag is configured differently"
  );
}

#[test]
fn it_works_with_flag_alias() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test flag alias"]).assert().success();
  doing.run(["flag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@flagged"),
    "entry should have @flagged via flag alias"
  );

  doing.run(["flag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@flagged"),
    "entry should not have @flagged after second flag toggle"
  );
}
