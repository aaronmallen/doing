use crate::support::helpers::DoingCmd;

#[test]
fn it_does_not_prompt_on_stdin() {
  let doing = DoingCmd::new();

  let output = doing
    .run(["--default", "now", "--section", "DefaultSection", "Default no prompt"])
    .output()
    .expect("failed to run doing");

  assert!(
    output.status.success(),
    "expected success with --default and piped stdin"
  );
}

#[test]
fn it_selects_default_option_in_menus() {
  let doing = DoingCmd::new();

  // Writing to a new section triggers a confirmation prompt;
  // --default should select the default option (yes for new sections)
  doing
    .run([
      "--default",
      "now",
      "--section",
      "DefaultSection",
      "Default choice entry",
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Default choice entry"),
    "expected entry to be created with --default, got: {contents}"
  );
}
