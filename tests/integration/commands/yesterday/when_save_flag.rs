use crate::support::helpers::DoingCmd;

#[test]
fn it_saves_output_to_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1d", "Yesterday save"]).assert().success();

  let save_path = doing.temp_dir_path().join("output.txt");
  let save_path_str = save_path.to_str().expect("valid path");

  let output = doing
    .run(["yesterday", "--save", save_path_str])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("Yesterday save"),
    "expected entry text in output, got: {stdout}"
  );
}
