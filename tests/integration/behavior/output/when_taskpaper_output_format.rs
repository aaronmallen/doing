use crate::support::helpers::DoingCmd;

#[test]
fn it_formats_entries_with_at_date_tags() {
  let doing = DoingCmd::new();
  doing.run(["now", "Taskpaper date test"]).assert().success();

  let output = doing
    .run(["show", "--output", "taskpaper"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // Ruby doing outputs entries with @date(YYYY-MM-DD HH:MM)
  assert!(
    stdout.contains("@date("),
    "expected @date() tag in taskpaper output, got: {stdout}"
  );
}

#[test]
fn it_preserves_user_tags() {
  let doing = DoingCmd::new();
  doing.run(["now", "Taskpaper tag test @project"]).assert().success();

  let output = doing
    .run(["show", "--output", "taskpaper"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("@project"),
    "expected user @project tag preserved in taskpaper output, got: {stdout}"
  );
}

#[test]
fn it_formats_sections_as_taskpaper_projects() {
  let doing = DoingCmd::new();
  doing.run(["now", "Taskpaper section test"]).assert().success();

  let output = doing
    .run(["show", "--output", "taskpaper"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  // In taskpaper format, sections are rendered as "Currently:" project headers
  // (though Ruby doing with -s filter may omit the header)
  // The entries themselves are prefixed with "- "
  assert!(
    stdout.contains("- "),
    "expected taskpaper entry prefix '- ', got: {stdout}"
  );
}

#[test]
fn it_includes_done_tag_for_finished_entries() {
  let doing = DoingCmd::new();
  doing.run(["now", "Taskpaper done test"]).assert().success();
  doing.run(["finish"]).assert().success();

  let output = doing
    .run(["show", "--output", "taskpaper"])
    .output()
    .expect("failed to run doing");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("@done"),
    "expected @done tag for finished entry in taskpaper output, got: {stdout}"
  );
}
