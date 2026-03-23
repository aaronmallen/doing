use crate::support::helpers::DoingCmd;

#[test]
fn it_reads_note_from_stdin() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry"]).assert().success();

  doing
    .cmd()
    .args(["note"])
    .write_stdin("Piped note text")
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Piped note text"),
    "expected note from stdin, got: {contents}"
  );
}
