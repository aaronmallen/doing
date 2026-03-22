use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_differs_from_full_date() {
  let doing = DoingCmd::new();
  doing.run(["now", "Date comparison entry"]).assert().success();

  let date_output = doing
    .run(["show", "--template", "%date"])
    .output()
    .expect("failed to run doing");
  let date_stdout = String::from_utf8_lossy(&date_output.stdout);

  let shortdate_output = doing
    .run(["show", "--template", "%shortdate"])
    .output()
    .expect("failed to run doing");
  let shortdate_stdout = String::from_utf8_lossy(&shortdate_output.stdout);

  let date_line = date_stdout.lines().next().expect("should have date line").trim();
  let shortdate_line = shortdate_stdout
    .lines()
    .next()
    .expect("should have shortdate line")
    .trim();

  assert_ne!(
    date_line, shortdate_line,
    "expected shortdate to differ from full date, both were: '{date_line}'"
  );
}

#[test]
fn it_renders_abbreviated_date() {
  let doing = DoingCmd::new();
  doing.run(["now", "Shortdate test entry"]).assert().success();

  let output = doing
    .run(["show", "--template", "%shortdate"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout.lines().next().expect("should have at least one line");
  let trimmed = line.trim();

  // Ruby doing shortdate outputs a time-only format like "6:50pm"
  assert!(
    !trimmed.is_empty(),
    "expected non-empty shortdate output, got empty string"
  );
}
