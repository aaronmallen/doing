use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_renders_correct_section_for_archived_entries() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[editors]
default = "cat"

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"
"#;
  let doing = DoingCmd::new_with_config(config);
  doing.run(["now", "Archive section test"]).assert().success();
  doing.run(["finish", "--archive"]).assert().success();

  let output = doing
    .run(["show", "--template", "%section: %title", "-s", "Archive"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Archive: Archive section test"),
    "expected 'Archive:' section prefix for archived entries, got: {stdout}"
  );
}

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_renders_section_name() {
  let doing = DoingCmd::new();
  doing.run(["now", "Section placeholder test"]).assert().success();

  let output = doing
    .run(["show", "--template", "%section: %title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("Currently: Section placeholder test"),
    "expected 'Currently: Section placeholder test' in output, got: {stdout}"
  );
}
