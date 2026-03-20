use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_deletes_matching_entries() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project alpha"]).assert().success();
  doing.run(["now", "Lunch break"]).assert().success();
  doing.run(["now", "Working on project beta"]).assert().success();

  doing.run(["grep", "--delete", "project"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(contents.contains("Lunch break"), "non-matching entry should remain");
  assert!(!contents.contains("project alpha"), "matching entry should be deleted");
  assert!(!contents.contains("project beta"), "matching entry should be deleted");
}

#[test]
fn it_does_not_duplicate_entries_matching_both_title_and_tag() {
  let doing = DoingCmd::new();

  doing.run(["now", "coding session @coding"]).assert().success();
  doing.run(["now", "Lunch break"]).assert().success();

  let output = doing.run(["grep", "coding"]).output().expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "entry matching both title and tag should appear only once"
  );
}

#[test]
fn it_finds_entries_by_tag_name() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project @coding"]).assert().success();
  doing.run(["now", "Lunch break"]).assert().success();
  doing.run(["now", "Meeting with team @planning"]).assert().success();

  let output = doing.run(["grep", "coding"]).output().expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "grep should find 1 entry by tag name"
  );
  assert!(stdout.contains("project"), "entry with @coding tag should be included");
}

#[test]
fn it_finds_entries_matching_search_term() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project alpha"]).assert().success();
  doing.run(["now", "Meeting about beta"]).assert().success();
  doing.run(["now", "Lunch break"]).assert().success();

  let output = doing.run(["grep", "project"]).output().expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 1, "grep should find 1 matching entry");
  assert!(stdout.contains("project alpha"), "matching entry should be included");
}

#[test]
fn it_outputs_search_results_as_json() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project alpha"]).assert().success();
  doing.run(["now", "Lunch break"]).assert().success();

  let output = doing
    .run(["grep", "project", "--output", "json"])
    .output()
    .expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert!(
    stdout.contains("project alpha"),
    "JSON output should contain the matching entry"
  );
  assert!(stdout.contains('{'), "output should be JSON formatted");
}

#[test]
fn it_produces_no_output_for_no_matches() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project"]).assert().success();

  let output = doing.run(["grep", "nonexistent"]).output().expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 0, "no entries should match");
}

#[test]
fn it_searches_across_all_sections() {
  let doing = DoingCmd::new();

  doing.run(["now", "Current coding task"]).assert().success();
  doing.run(["done", "Finished coding task"]).assert().success();

  let output = doing
    .run(["grep", "coding", "--section", "All"])
    .output()
    .expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "grep with --section All should find entries across all sections"
  );
}

#[test]
fn it_searches_with_exact_mode() {
  let doing = DoingCmd::new();

  doing.run(["now", "The barley is ripe"]).assert().success();
  doing.run(["now", "barley hoop game"]).assert().success();
  doing.run(["now", "Something else entirely"]).assert().success();

  let output = doing
    .run(["grep", "'barley hoop"])
    .output()
    .expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "exact search with ' prefix should match only the entry with the exact substring"
  );
  assert!(
    stdout.contains("barley hoop game"),
    "exact substring match should be included"
  );
}

#[test]
fn it_searches_with_fuzzy_mode() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project"]).assert().success();
  doing.run(["now", "Meeting notes"]).assert().success();

  let output = doing
    .run(["grep", "--fuzzy", "wrking"])
    .output()
    .expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(helpers::count_entries(&stdout), 1, "fuzzy search should match 1 entry");
  assert!(
    stdout.contains("Working on project"),
    "fuzzy match should find the entry"
  );
}

#[test]
fn it_searches_with_pattern_mode_include_exclude() {
  let doing = DoingCmd::new();

  doing.run(["now", "Alpha project coding"]).assert().success();
  doing.run(["now", "Beta project review"]).assert().success();
  doing.run(["now", "Gamma coding review"]).assert().success();

  let output = doing
    .run(["grep", "+coding -review"])
    .output()
    .expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "pattern +coding -review should match 1 entry"
  );
  assert!(
    stdout.contains("Alpha"),
    "entry with coding but not review should be included"
  );
}

#[test]
fn it_searches_with_pattern_mode_quoted_phrase() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project alpha"]).assert().success();
  doing.run(["now", "Project alpha is great"]).assert().success();
  doing.run(["now", "Alpha project beta"]).assert().success();

  let output = doing
    .run(["grep", "\"project alpha\""])
    .output()
    .expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "quoted phrase search should match entries containing the exact phrase"
  );
}

#[test]
fn it_searches_with_regex_mode() {
  let doing = DoingCmd::new();

  doing.run(["now", "Bug fix 123"]).assert().success();
  doing.run(["now", "Bug fix 456"]).assert().success();
  doing.run(["now", "Feature work"]).assert().success();

  let output = doing.run(["grep", "/fix \\d+/"]).output().expect("failed to run grep");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "regex search should match 2 entries with 'fix' followed by digits"
  );
  assert!(
    !stdout.contains("Feature work"),
    "non-matching entry should be excluded"
  );
}

#[test]
fn it_works_with_search_alias() {
  let doing = DoingCmd::new();

  doing.run(["now", "Working on project alpha"]).assert().success();
  doing.run(["now", "Lunch break"]).assert().success();

  let output = doing.run(["search", "project"]).output().expect("failed to run search");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "search alias should find 1 matching entry"
  );
  assert!(stdout.contains("project alpha"), "matching entry should be included");
}
