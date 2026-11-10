use crate::support::helpers::DoingCmd;

const CONFIG_WITH_TEMPLATE: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[editors]
default = "cat"

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[templates.custom]
date_format = "%Y-%m-%d"
template = "%date :: %title"
wrap_width = 0
order = "asc"
"#;

#[test]
fn it_uses_config_template() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_TEMPLATE);

  doing.run(["now", "Config template test"]).assert().success();

  let output = doing
    .run(["today", "--config-template", "custom"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Config template test"),
    "expected entry in output, got: {stdout}"
  );
}
