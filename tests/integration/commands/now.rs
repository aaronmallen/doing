mod when_back_flag_is_provided;
mod when_finish_last_flag_is_provided;
mod when_from_flag_is_provided;
mod when_multiple_flags_are_combined;
mod when_no_flags_are_provided;
mod when_noauto_flag_is_provided;
mod when_note_flag_is_provided;
mod when_section_flag_is_provided;

use crate::support::helpers::DoingCmd;

#[test]
fn it_works_as_next_alias() {
  let doing = DoingCmd::new();

  doing.run(["next", "Next alias test"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Next alias test"),
    "expected 'next' alias to create entry, got: {contents}"
  );
}
