use crate::support::helpers::DoingCmd;

#[test]
fn it_exits_successfully() {
  let doing = DoingCmd::new();

  doing.run(["colors"]).assert().success();
}

#[test]
fn it_lists_available_color_variables() {
  let doing = DoingCmd::new();

  let output = doing.run(["colors"]).output().expect("failed to run colors command");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Brett's doing colors outputs color variable names like "black", "red", "green", etc.
  assert!(stdout.contains("red"), "expected 'red' in colors output, got: {stdout}");
  assert!(
    stdout.contains("green"),
    "expected 'green' in colors output, got: {stdout}"
  );
  assert!(
    stdout.contains("blue"),
    "expected 'blue' in colors output, got: {stdout}"
  );
  assert!(
    stdout.contains("yellow"),
    "expected 'yellow' in colors output, got: {stdout}"
  );
  assert!(
    stdout.contains("white"),
    "expected 'white' in colors output, got: {stdout}"
  );
  assert!(
    stdout.contains("black"),
    "expected 'black' in colors output, got: {stdout}"
  );
}

#[test]
fn it_outputs_non_empty_list() {
  let doing = DoingCmd::new();

  let output = doing.run(["colors"]).output().expect("failed to run colors command");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let non_empty_lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
  assert!(
    !non_empty_lines.is_empty(),
    "expected at least one color variable in output, got empty"
  );
}
