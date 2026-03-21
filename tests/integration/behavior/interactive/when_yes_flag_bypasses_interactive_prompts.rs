use crate::support::helpers::DoingCmd;

#[test]
fn it_skips_confirmation_prompt_on_cancel() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry to cancel"]).assert().success();

  // --yes should bypass any confirmation prompt
  doing.run(["--yes", "cancel"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@done"),
    "expected entry to be cancelled with --yes, got: {contents}"
  );
}

#[test]
fn it_skips_confirmation_prompt_on_reset() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry to reset"]).assert().success();

  // --yes should bypass any confirmation prompt
  doing.run(["--yes", "reset"]).assert().success();
}
