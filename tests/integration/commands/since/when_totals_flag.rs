use crate::support::helpers::DoingCmd;

#[test]
fn it_shows_tag_time_totals() {
  let doing = DoingCmd::new();

  doing.run(["now", "--back", "1h", "Since @project"]).assert().success();
  doing.run(["done"]).assert().success();

  let output = doing
    .run(["since", "2h ago", "--totals"])
    .output()
    .expect("failed to run");

  assert!(output.status.success(), "expected success exit code");
}
