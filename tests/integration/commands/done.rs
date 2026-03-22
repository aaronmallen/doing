mod when_archive_flag_is_provided;
mod when_at_flag_is_provided;
mod when_back_flag_is_provided;
mod when_entry_text_is_provided;
mod when_from_flag_is_provided;
mod when_multiple_flags_are_combined;
mod when_no_argument_is_provided;
mod when_no_date_flag_is_provided;
mod when_noauto_flag_is_provided;
mod when_note_flag_is_provided;
mod when_remove_flag_is_provided;
mod when_section_flag_is_provided;
mod when_took_flag_is_provided;
mod when_unfinished_flag_is_provided;

use crate::support::helpers::DoingCmd;

#[test]
fn it_works_as_did_alias() {
  let doing = DoingCmd::new();

  doing.run(["did", "Did alias test"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("Did alias test"),
    "expected 'did' alias to create entry, got: {contents}"
  );
  assert!(
    contents.contains("@done("),
    "expected @done tag on entry created via 'did' alias, got: {contents}"
  );
}
