use crate::support::helpers::DoingCmd;

#[test]
fn it_accepts_pager_flag_without_error() {
  let doing = DoingCmd::new();
  doing.run(["now", "Pager flag test"]).assert().success();

  doing.run(["--pager", "show"]).assert().success();
}

#[test]
fn it_accepts_short_form_flag() {
  let doing = DoingCmd::new();
  doing.run(["now", "Pager short flag test"]).assert().success();

  doing.run(["-p", "show"]).assert().success();
}
