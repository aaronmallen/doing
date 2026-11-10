mod when_back_flag_is_provided;
mod when_bool_flag_is_and;
mod when_bool_flag_is_not;
mod when_bool_flag_is_or;
mod when_bool_flag_is_pattern;
mod when_case_flag_is_provided;
mod when_exact_flag_is_provided;
mod when_in_flag_is_provided;
mod when_multiple_flags_are_combined;
mod when_no_flags_are_provided;
mod when_noauto_flag_is_provided;
mod when_not_flag_is_provided;
mod when_note_flag_is_provided;
mod when_search_flag_is_provided;
mod when_section_flag_is_provided;
mod when_tag_flag_is_provided;
mod when_val_flag_is_provided;

use std::fs;

use crate::support::helpers::DoingCmd;

#[test]
fn it_works_as_resume_alias() {
  let doing = DoingCmd::new();

  fs::write(
    doing.doing_file_path(),
    "Currently:\n\t- 2024-01-10 10:00 | Resume test task <aaa111>\n",
  )
  .expect("failed to write doing file");

  doing.run(["resume"]).assert().success();

  let contents = doing.read_doing_file();

  // Should have 2 entries - original with @done and new duplicate
  let entry_count = contents.matches("Resume test task").count();
  assert!(
    entry_count >= 2,
    "expected 'resume' alias to duplicate entry, got {entry_count} occurrences in: {contents}"
  );
}
