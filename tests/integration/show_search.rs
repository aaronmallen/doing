use pretty_assertions::assert_eq;

use crate::helpers::{self, DoingCmd};

#[test]
fn it_combines_search_with_date_sorting() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "3h ago", "Alpha searchable"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "2h ago", "Beta searchable"])
    .assert()
    .success();
  doing
    .run(["now", "--back", "1h ago", "Gamma searchable"])
    .assert()
    .success();
  doing.run(["now", "Delta other"]).assert().success();

  let output = doing
    .run(["show", "--search", "searchable", "--sort", "asc"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().filter(|l| l.contains('|')).collect();

  assert_eq!(lines.len(), 3, "search + sort should return 3 matching entries");
  assert!(lines[0].contains("Alpha"), "first entry in asc sort should be Alpha");
  assert!(lines[1].contains("Beta"), "second entry in asc sort should be Beta");
  assert!(lines[2].contains("Gamma"), "third entry in asc sort should be Gamma");
  assert!(!stdout.contains("Delta"), "non-matching entry should be excluded");
}

#[test]
fn it_filters_entries_by_search() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry unique string"]).assert().success();
  doing.run(["now", "Test entry barley hoop"]).assert().success();
  doing.run(["now", "Test entry Barley hooP"]).assert().success();

  let output = doing
    .run(["show", "--search", "barley hoop"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "case-insensitive search should match 2 entries"
  );
  assert!(
    !stdout.contains("unique string"),
    "non-matching entry should be excluded"
  );
}

#[test]
fn it_negates_search_results_with_not_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry unique string"]).assert().success();
  doing.run(["now", "Test entry barley hoop"]).assert().success();
  doing.run(["now", "Test entry Barley hooP"]).assert().success();

  let output = doing
    .run(["show", "--search", "barley", "--not"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "negated search should exclude 2 barley entries"
  );
  assert!(stdout.contains("unique string"), "non-matching entry should remain");
}

#[test]
#[ignore = "show command missing --case flag (see plan to add --case to FilterArgs)"]
fn it_performs_case_sensitive_search_with_case_flag() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry barley hoop"]).assert().success();
  doing.run(["now", "Test entry Barley hooP"]).assert().success();

  let output = doing
    .run(["show", "--search", "barley hoop", "--case", "sensitive"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "case-sensitive search should match only the lowercase entry"
  );
  assert!(stdout.contains("barley hoop"), "exact case match should be included");
}

#[test]
fn it_performs_case_sensitive_search_with_smart_case() {
  let doing = DoingCmd::new();

  doing.run(["now", "Test entry barley hoop"]).assert().success();
  doing.run(["now", "Test entry Barley hooP"]).assert().success();

  let output = doing
    .run(["show", "--search", "Barley hooP"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "smart case with uppercase should match case-sensitively"
  );
  assert!(stdout.contains("Barley hooP"), "exact case match should be included");
}

#[test]
fn it_searches_with_exact_mode() {
  let doing = DoingCmd::new();

  doing.run(["now", "The barley is ripe"]).assert().success();
  doing.run(["now", "barley hoop game"]).assert().success();
  doing.run(["now", "Something else entirely"]).assert().success();

  let output = doing
    .run(["show", "--search", "'barley hoop"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "exact search should match only the entry with the exact substring"
  );
  assert!(
    stdout.contains("barley hoop game"),
    "exact substring match should be included"
  );
}

#[test]
fn it_searches_with_pattern_mode_include_exclude() {
  let doing = DoingCmd::new();

  doing.run(["now", "Alpha project coding"]).assert().success();
  doing.run(["now", "Beta project review"]).assert().success();
  doing.run(["now", "Gamma coding review"]).assert().success();

  let output = doing
    .run(["show", "--search", "+coding -review"])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    1,
    "pattern search +coding -review should match 1 entry"
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
    .run(["show", "--search", "\"project alpha\""])
    .output()
    .expect("failed to run show");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_eq!(
    helpers::count_entries(&stdout),
    2,
    "quoted phrase search should match entries containing the exact phrase"
  );
  assert!(
    stdout.contains("Working on project alpha"),
    "entry with exact phrase should be included"
  );
  assert!(
    stdout.contains("Project alpha is great"),
    "entry with exact phrase (case-insensitive) should be included"
  );
}

#[test]
fn it_searches_with_regex_mode() {
  let doing = DoingCmd::new();

  doing.run(["now", "Bug fix 123"]).assert().success();
  doing.run(["now", "Bug fix 456"]).assert().success();
  doing.run(["now", "Feature work"]).assert().success();

  let output = doing
    .run(["show", "--search", "/fix \\d+/"])
    .output()
    .expect("failed to run show");
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
