use crate::support::helpers::DoingCmd;

#[test]
fn it_excludes_auto_tags() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[autotag]
coding = "dev"

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

  let doing = DoingCmd::new_with_config(config);

  // Without -x, autotag should apply ("coding" in title adds @dev tag)
  doing.run(["done", "coding session"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@dev"),
    "expected @dev autotag without -x, got: {contents}"
  );

  // With -x, autotag should NOT apply
  let doing2 = DoingCmd::new_with_config(config);

  doing2.run(["done", "-x", "coding session"]).assert().success();

  let contents2 = doing2.read_doing_file();
  assert!(
    !contents2.contains("@dev"),
    "expected no @dev autotag with -x, got: {contents2}"
  );
}
