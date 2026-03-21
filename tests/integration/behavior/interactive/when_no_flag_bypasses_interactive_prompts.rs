use crate::support::helpers::DoingCmd;

#[test]
fn it_declines_confirmation_on_reset() {
  let doing = DoingCmd::new();

  doing.run(["now", "Entry that should stay"]).assert().success();

  let before = doing.read_doing_file();

  // --no should decline any confirmation, leaving the entry unchanged
  let output = doing
    .run(["--no", "reset"])
    .output()
    .expect("failed to run doing --no reset");

  // The command completes (doesn't hang) — that's the key assertion
  let _ = output.status;

  let after = doing.read_doing_file();
  assert_eq!(before, after, "expected doing file to remain unchanged with --no");
}
