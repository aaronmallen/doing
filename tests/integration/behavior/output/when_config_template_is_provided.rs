use crate::support::helpers::DoingCmd;

#[test]
fn it_uses_named_template_from_config() {
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

[templates.custom]
date_format = "%Y-%m-%d"
template = "CUSTOM: %title"
wrap_width = 0
order = "asc"
"#;
  let doing = DoingCmd::new_with_config(config);
  doing.run(["now", "Config template test"]).assert().success();

  let output = doing
    .run(["show", "--config-template", "custom"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("CUSTOM: Config template test"),
    "expected custom template format, got: {stdout}"
  );
}

#[test]
fn it_falls_back_to_default_template_if_name_not_found() {
  let doing = DoingCmd::new();
  doing.run(["now", "Config template fallback test"]).assert().success();

  let output = doing
    .run(["show", "--config-template", "nonexistent"])
    .output()
    .expect("failed to run doing");

  // Ruby doing returns error: "Unknown template: nonexistent" with exit 1
  assert!(
    !output.status.success(),
    "expected failure for unknown config template name"
  );
}
