use crate::support::helpers::DoingCmd;

#[test]
fn it_saves_output_to_file() {
  let doing = DoingCmd::new();

  doing.run(["now", "Save test entry"]).assert().success();

  let save_path = doing.temp_dir_path().join("output.txt");
  let save_path_str = save_path.to_str().expect("valid path");

  let output = doing
    .run(["today", "--save", save_path_str])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
