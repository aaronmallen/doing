use crate::helpers::DoingCmd;

const BUDGET_CONFIG: &str = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false

[budgets]
dev = "10h"
meetings = "5h"

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

fn doing_with_budgets() -> DoingCmd {
  DoingCmd::new_with_config(BUDGET_CONFIG)
}

#[test]
fn it_accepts_tag_first_then_amount() {
  let doing = doing_with_budgets();

  doing.run(["budget", "coding", "50h"]).assert().success();
}

#[test]
fn it_errors_for_unconfigured_tag() {
  let doing = doing_with_budgets();

  doing.run(["budget", "nonexistent"]).assert().failure();
}

#[test]
fn it_lists_all_budgets_with_no_args() {
  let doing = doing_with_budgets();

  doing.run(["now", "working on stuff @dev"]).assert().success();

  doing
    .run(["budget"])
    .assert()
    .success()
    .stdout(predicates::str::contains("dev:"))
    .stdout(predicates::str::contains("meetings:"));
}

#[test]
fn it_shows_single_tag_budget() {
  let doing = doing_with_budgets();

  doing.run(["now", "working on stuff @dev"]).assert().success();

  doing
    .run(["budget", "dev"])
    .assert()
    .success()
    .stdout(predicates::str::contains("dev:"));
}
