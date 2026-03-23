use crate::support::helpers::DoingCmd;

const CONFIG_WITH_CUSTOM_TEMPLATE: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[interaction]
confirm_longer_than = ""

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[templates.custom]
date_format = "%Y-%m-%d"
template = "%date | %title"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

#[test]
fn it_uses_config_template() {
  let doing = DoingCmd::new_with_config(CONFIG_WITH_CUSTOM_TEMPLATE);

  doing.run(["now", "Config template test"]).assert().success();

  let output = doing
    .run(["recent", "--config-template", "custom"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
