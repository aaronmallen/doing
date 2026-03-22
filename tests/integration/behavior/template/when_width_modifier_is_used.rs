use crate::support::helpers::DoingCmd;

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_pads_output_to_specified_width() {
  let doing = DoingCmd::new();
  doing.run(["now", "Short"]).assert().success();

  let output = doing
    .run(["show", "--template", "%80title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout.lines().next().expect("should have at least one line");

  // "Short" is 5 chars, with %80title it should be padded to 80 chars
  assert!(
    line.len() >= 80,
    "expected line padded to at least 80 chars, got {} chars: '{line}'",
    line.len()
  );
}

#[test]
#[ignore = "requires --template inline string support (see #158)"]
fn it_truncates_long_titles_to_width() {
  let doing = DoingCmd::new();
  let long_title = "A".repeat(100);
  doing.run(["now", &long_title]).assert().success();

  let output = doing
    .run(["show", "--template", "%40title"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains('A'))
    .expect("should find entry line");

  // With %40title, the title portion should be truncated/limited to around 40 chars
  assert!(
    line.trim().len() <= 40,
    "expected line truncated to ~40 chars, got {} chars: '{line}'",
    line.trim().len()
  );
}
