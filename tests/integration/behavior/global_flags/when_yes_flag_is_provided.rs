use crate::support::helpers::DoingCmd;

#[test]
fn it_auto_confirms_destructive_operations() {
  let doing = DoingCmd::new();

  // Writing to a new section triggers a confirmation prompt;
  // --yes should auto-confirm it
  doing
    .run(["--yes", "now", "--section", "NewSection", "Auto confirmed entry"])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Auto confirmed entry"),
    "expected entry to be created with --yes, got: {contents}"
  );
}

#[test]
fn it_does_not_prompt_on_stdin() {
  let doing = DoingCmd::new();

  // With piped input (no tty) and --yes, should complete without hanging
  let output = doing
    .run(["--yes", "now", "--section", "AnotherSection", "No prompt test"])
    .output()
    .expect("failed to run doing");

  assert!(output.status.success(), "expected success with --yes and piped stdin");
}
