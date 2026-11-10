mod when_basic_search;
mod when_bool_flag;
mod when_case_flag;
mod when_delete_flag;
mod when_duration_flag;
mod when_exact_flag;
mod when_from_flag;
mod when_not_flag;
mod when_only_timed_flag;
mod when_output_flag;
mod when_section_flag;
mod when_tag_flag;
mod when_template_flag;
mod when_times_flag;
mod when_title_flag;
mod when_totals_flag;
mod when_val_flag;

use crate::support::helpers::DoingCmd;

#[test]
fn it_is_accessible_via_search_alias() {
  let doing = DoingCmd::new();

  doing.run(["now", "Search alias test entry"]).assert().success();

  let grep_output = doing
    .run(["grep", "Search alias"])
    .output()
    .expect("failed to run grep");
  let search_output = doing
    .run(["search", "Search alias"])
    .output()
    .expect("failed to run search");

  let grep_stdout = String::from_utf8_lossy(&grep_output.stdout);
  let search_stdout = String::from_utf8_lossy(&search_output.stdout);

  assert_eq!(
    grep_stdout, search_stdout,
    "expected 'search' alias to produce same output as 'grep'"
  );
}
