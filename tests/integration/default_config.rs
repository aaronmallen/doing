use crate::helpers::{self, DoingCmd};

/// Minimal config with NO explicit template — exercises the built-in default.
const BARE_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[editors]
default = "cat"
"#;

#[test]
fn it_displays_entries_with_builtin_default_template() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing.run(["now", "Default template entry"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "show should display 1 entry when using the built-in default template"
  );
  assert!(
    stdout.contains("Default template entry"),
    "output should contain the entry title"
  );
}

#[test]
fn it_displays_multiple_entries_with_builtin_default_template() {
  let doing = DoingCmd::new_with_config(BARE_CONFIG);

  doing.run(["now", "First entry"]).assert().success();
  doing.run(["now", "Second entry @coding"]).assert().success();

  let output = doing.run(["show"]).output().expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "show should display 2 entries when using the built-in default template"
  );
  assert!(stdout.contains("First entry"), "output should contain the first entry");
  assert!(
    stdout.contains("Second entry"),
    "output should contain the second entry"
  );
}
