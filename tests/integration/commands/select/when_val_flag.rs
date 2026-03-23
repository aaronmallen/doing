use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_filters_by_tag_value() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "\
Currently:
\t- 2024-01-10 10:00 | ClientA task @project(clientA) <aaa111>
\t- 2024-01-11 10:00 | ClientB task @project(clientB) <bbb222>
",
  )
  .expect("failed to write doing file");

  doing
    .run([
      "select",
      "--no-menu",
      "--tag",
      "project",
      "--val",
      "clientA",
      "--delete",
    ])
    .assert()
    .success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("ClientA"),
    "expected clientA entry to be deleted, got: {contents}"
  );
  assert!(
    contents.contains("ClientB"),
    "expected clientB entry to remain, got: {contents}"
  );
}
