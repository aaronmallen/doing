use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_adds_note_to_new_entry() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Task with note <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["again", "--note", "Additional context"]).assert().success();

  let contents = doing.read_doing_file();

  assert!(
    contents.contains("Additional context"),
    "expected note to be added to new entry, got: {contents}"
  );
}
