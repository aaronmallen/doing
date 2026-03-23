use crate::support::helpers::DoingCmd;

#[test]
fn it_excludes_auto_tags_and_default_tags() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[autotag.transforms.testing]
filter = ".*"
tags = ["autotag"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

  let doing = DoingCmd::new_with_config(config);

  // With -x flag, autotag rules should not apply
  doing.run(["now", "-x", "Noauto entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Noauto entry"),
    "expected entry to be created, got: {contents}"
  );
  assert!(
    !contents.contains("@autotag"),
    "expected autotag to NOT be applied with -X flag, got: {contents}"
  );
}
