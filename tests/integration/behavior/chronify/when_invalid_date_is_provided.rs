use crate::support::helpers::DoingCmd;

#[test]
fn it_rejects_empty_string() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "", "empty back entry"])
    .assert()
    .failure()
    .stderr(predicates::str::contains("invalid time expression"));
}

#[test]
fn it_rejects_invalid_date_values() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "13/32/2026", "invalid date entry"])
    .assert()
    .failure();
}

#[test]
fn it_rejects_nonsense_string() {
  let doing = DoingCmd::new();

  doing
    .run(["now", "--back", "not a date at all", "nonsense entry"])
    .assert()
    .failure();
}
