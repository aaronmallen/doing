use predicates::prelude::*;

use crate::helpers::DoingCmd;

#[test]
fn it_displays_latest_changelog_by_default() {
  let doing = DoingCmd::new();

  doing
    .run(["changes"])
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}

#[test]
fn it_displays_all_versions_with_all_flag() {
  let doing = DoingCmd::new();

  doing
    .run(["changes", "--all"])
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}

#[test]
fn it_outputs_raw_markdown() {
  let doing = DoingCmd::new();

  doing
    .run(["changes", "--markdown"])
    .assert()
    .success()
    .stdout(predicate::str::contains("###"));
}

#[test]
fn it_outputs_changes_only() {
  let doing = DoingCmd::new();

  // --changes should not include version headers
  doing
    .run(["changes", "-C"])
    .assert()
    .success()
    .stdout(predicate::str::contains("alpha").not());
}

#[test]
fn it_filters_by_search_term() {
  let doing = DoingCmd::new();

  doing
    .run(["changes", "--all", "-s", "pager"])
    .assert()
    .success()
    .stdout(predicate::str::contains("pager"));
}

#[test]
fn it_works_with_changelog_alias() {
  let doing = DoingCmd::new();

  doing
    .run(["changelog"])
    .assert()
    .success()
    .stdout(predicate::str::is_empty().not());
}
