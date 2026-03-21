use crate::helpers::DoingCmd;

#[test]
fn it_limits_results_with_max_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @beta"]).assert().success();
  doing.run(["now", "Task @gamma"]).assert().success();

  let output = doing.run(["tags", "2"]).output().expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert_eq!(lines.len(), 2, "should limit output to 2 tags");
}

#[test]
fn it_lists_all_tags_with_counts() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project @coding"]).assert().success();
  doing.run(["now", "Meeting about design @meeting"]).assert().success();
  doing.run(["now", "Review code @coding @review"]).assert().success();

  let output = doing.run(["tags", "--counts"]).output().expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("@coding"), "tags should list '@coding'");
  assert!(stdout.contains("@meeting"), "tags should list '@meeting'");
  assert!(stdout.contains("@review"), "tags should list '@review'");
  assert!(stdout.contains("(2)"), "coding should appear twice");
}

#[test]
fn it_outputs_line_format_with_at_prefix() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @doing"]).assert().success();
  doing.run(["done", "Finished @done-task"]).assert().success();

  let output = doing.run(["tags", "--line"]).output().expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(stdout.contains("@do"), "line output should include @-prefixed tags");
}

#[test]
fn it_scopes_tags_to_section() {
  let doing = DoingCmd::new();

  doing.run(["now", "Current task @coding"]).assert().success();
  doing
    .run(["now", "--section", "Archive", "Old task @writing"])
    .assert()
    .success();

  let output = doing
    .run(["tags", "--section", "Currently"])
    .output()
    .expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("@coding"),
    "section-scoped tags should include '@coding'"
  );
  assert!(
    !stdout.contains("@writing"),
    "section-scoped tags should not include tags from other sections"
  );
}

#[test]
fn it_shows_tag_names_and_counts() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one @project"]).assert().success();
  doing.run(["now", "Task two @project"]).assert().success();
  doing.run(["now", "Task three @project"]).assert().success();

  let output = doing.run(["tags", "--counts"]).output().expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("@project"),
    "output should contain the tag name with @ prefix"
  );
  assert!(stdout.contains("(3)"), "output should show the count of 3");
}

#[test]
fn it_sorts_by_count() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one @rare"]).assert().success();
  doing.run(["now", "Task two @common"]).assert().success();
  doing.run(["now", "Task three @common"]).assert().success();
  doing.run(["now", "Task four @common"]).assert().success();
  doing.run(["now", "Task five @medium"]).assert().success();
  doing.run(["now", "Task six @medium"]).assert().success();

  let output = doing
    .run(["tags", "--sort", "count", "--order", "desc", "--counts"])
    .output()
    .expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(
    lines[0].contains("@common"),
    "first tag in desc count sort should be common"
  );
  assert!(
    lines[lines.len() - 1].contains("@rare"),
    "last tag in desc count sort should be rare"
  );
}

#[test]
fn it_sorts_tags_alphabetically() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task @zebra"]).assert().success();
  doing.run(["now", "Task @alpha"]).assert().success();
  doing.run(["now", "Task @middle"]).assert().success();

  let output = doing
    .run(["tags", "--sort", "name", "--order", "asc"])
    .output()
    .expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  assert!(lines[0].contains("@alpha"), "first tag in asc sort should be alpha");
  assert!(
    lines[lines.len() - 1].contains("@zebra"),
    "last tag in asc sort should be zebra"
  );
}

#[test]
fn it_uses_c_short_flag_for_counts() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task one @coding"]).assert().success();
  doing.run(["now", "Task two @coding"]).assert().success();

  let output = doing.run(["tags", "-c"]).output().expect("failed to run tags");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("@coding") && stdout.contains('('),
    "-c should show counts like --counts, got: {stdout}"
  );
}
