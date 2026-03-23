use crate::support::helpers::DoingCmd;

const AUTOTAG_CONFIG: &str = r#"
current_section = "Currently"
default_tags = ["tracked"]
doing_file_sort = "asc"
include_notes = true
paginate = false

[autotag]
whitelist = ["project"]
transform = []

[autotag.synonyms]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

#[test]
fn it_accepts_short_form_x() {
  let doing = DoingCmd::new_with_config(AUTOTAG_CONFIG);

  doing.run(["-x", "now", "Short form test"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@tracked"),
    "expected no @tracked tag with -x, got: {contents}"
  );
}

#[test]
fn it_excludes_autotag_matches() {
  let doing = DoingCmd::new_with_config(AUTOTAG_CONFIG);

  doing
    .run(["now", "-x", "Working on project feature"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@project"),
    "expected no @project autotag with --noauto, got: {contents}"
  );
}

#[test]
fn it_excludes_default_tags() {
  let doing = DoingCmd::new_with_config(AUTOTAG_CONFIG);

  doing.run(["now", "-x", "No default tags entry"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@tracked"),
    "expected no @tracked default tag with --noauto, got: {contents}"
  );
}

#[test]
fn it_preserves_explicitly_typed_tags() {
  let doing = DoingCmd::new_with_config(AUTOTAG_CONFIG);

  doing.run(["now", "-x", "Entry with @manual tag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@manual"),
    "expected explicit @manual tag preserved with --noauto, got: {contents}"
  );
  assert!(
    !contents.contains("@tracked"),
    "expected no @tracked default tag with --noauto, got: {contents}"
  );
}
