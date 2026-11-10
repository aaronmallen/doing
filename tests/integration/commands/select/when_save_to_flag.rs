use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_saves_output_to_file() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Saved task <aaa111>\n",
  )
  .expect("failed to write doing file");

  let output_path = doing.temp_dir_path().join("output.txt");

  doing
    .run([
      "select",
      "--no-menu",
      "--query",
      "Saved task",
      "--save-to",
      output_path.to_str().unwrap(),
    ])
    .assert()
    .success();

  let saved_contents = fs::read_to_string(&output_path).expect("failed to read saved file");
  assert!(
    saved_contents.contains("Saved task"),
    "expected saved file to contain entry, got: {saved_contents}"
  );
}
