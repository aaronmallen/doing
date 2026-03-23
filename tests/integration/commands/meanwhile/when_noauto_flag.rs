use crate::support::helpers::DoingCmd;

#[test]
fn it_disables_autotag() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false
default_tags = ["defaulttag"]

[autotag]
MW = "autotagged"

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

  // Without --noauto, default tags should apply
  let doing = DoingCmd::new_with_config(config);

  doing.run(["meanwhile", "MW with autotags"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    contents.contains("@defaulttag"),
    "expected @defaulttag without --noauto, got: {contents}"
  );

  // With --noauto, default tags should NOT apply
  let doing2 = DoingCmd::new_with_config(config);

  doing2
    .run(["meanwhile", "--noauto", "MW without autotags"])
    .assert()
    .success();

  let contents2 = doing2.read_doing_file();
  assert!(
    !contents2.contains("@defaulttag"),
    "expected no @defaulttag with --noauto, got: {contents2}"
  );
}

#[test]
fn it_disables_autotag_with_short_flag() {
  let config = r#"
current_section = "Currently"
doing_file_sort = "asc"
include_notes = true
paginate = false
default_tags = ["defaulttag"]

[templates.default]
date_format = "%Y-%m-%d %H:%M"
template = "%date | %title%note"
wrap_width = 0
order = "asc"

[editors]
default = "cat"
"#;

  let doing = DoingCmd::new_with_config(config);

  doing.run(["meanwhile", "-x", "MW with X flag"]).assert().success();

  let contents = doing.read_doing_file();
  assert!(
    !contents.contains("@defaulttag"),
    "expected no @defaulttag with -X, got: {contents}"
  );
}
