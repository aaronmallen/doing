mod when_after_flag_is_provided;
mod when_before_flag_is_provided;
mod when_case_flag_is_provided;
mod when_exact_flag_is_provided;
mod when_from_flag_is_provided;
mod when_keep_flag_is_provided;
mod when_multiple_flags_are_combined;
mod when_no_flags_are_provided;
mod when_no_label_flag_is_provided;
mod when_not_flag_is_provided;
mod when_search_flag_is_provided;
mod when_section_argument_is_provided;
mod when_tag_flag_is_provided;
mod when_to_flag_is_provided;
mod when_val_flag_is_provided;

use crate::support::helpers::DoingCmd;

#[test]
fn it_works_as_move_alias() {
  let doing = DoingCmd::new();

  doing.run(["now", "Task for move alias"]).assert().success();
  doing.run(["move"]).assert().success();

  let contents = doing.read_doing_file();

  // Entry should not be in Currently section
  let currently_section = contents.split("Archive:").next().unwrap_or("");
  assert!(
    !currently_section.contains("Task for move alias"),
    "expected entry to be moved out of Currently, got: {contents}"
  );

  // Entry should be in Archive section
  let archive_section = contents.split("Archive:").nth(1).unwrap_or("");
  assert!(
    archive_section.contains("Task for move alias"),
    "expected entry to be in Archive section after 'move', got: {contents}"
  );
}
