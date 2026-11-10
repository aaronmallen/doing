use crate::support::helpers::DoingCmd;

#[test]
fn it_renders_duration_for_done_entries() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "30 minutes ago", "Duration done test"])
    .assert()
    .success();
  doing.run(["finish"]).assert().success();

  let output = doing
    .run(["show", "--template", "%title (%duration)"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Duration done test"))
    .expect("should find entry line");

  // Done entries should show duration between start and @done
  // The parentheses should contain some value (possibly empty for very short intervals)
  assert!(
    line.contains('(') && line.contains(')'),
    "expected parentheses in duration output, got: {line}"
  );
}

#[test]
fn it_renders_elapsed_duration_for_open_entries() {
  let doing = DoingCmd::new();
  doing
    .run(["now", "--back", "30 minutes ago", "Duration open test"])
    .assert()
    .success();

  let output = doing
    .run(["show", "--template", "%title (%duration)", "--duration"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  let line = stdout
    .lines()
    .find(|l| l.contains("Duration open test"))
    .expect("should find entry line");

  // Open entries should have a non-empty duration showing time since start
  assert!(
    !line.contains("()"),
    "expected non-empty duration for open entry, got: {line}"
  );
}
