use crate::support::helpers::DoingCmd;

#[test]
fn it_auto_declines_confirmations() {
  let doing = DoingCmd::new();

  // Writing to a new section triggers a confirmation prompt;
  // --no should auto-decline it, so the entry is NOT created
  let output = doing
    .run(["--no", "now", "--section", "DeclinedSection", "Should not appear"])
    .output()
    .expect("failed to run doing");

  assert!(!output.status.success(), "expected non-zero exit when --no declines");

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("Should not appear"),
    "expected entry NOT to be created with --no, got: {contents}"
  );
}

#[test]
fn it_does_not_prompt_on_stdin() {
  let doing = DoingCmd::new();

  // With --no and piped input, should complete without hanging (even if it fails)
  let output = doing
    .run(["--no", "now", "--section", "NoPromptSection", "No prompt decline"])
    .output()
    .expect("failed to run doing");

  // The command completes (doesn't hang) — that's the key assertion
  assert!(
    !output.status.success(),
    "expected non-zero exit when --no declines section creation"
  );
}
