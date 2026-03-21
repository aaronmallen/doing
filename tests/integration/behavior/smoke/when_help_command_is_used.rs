use crate::support::helpers::DoingCmd;

macro_rules! help_test {
  ($command:ident) => {
    #[test]
    fn $command() {
      let doing = DoingCmd::new();

      doing
        .run([stringify!($command), "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"));
    }
  };
  ($test_name:ident, $command:literal) => {
    #[test]
    fn $test_name() {
      let doing = DoingCmd::new();

      doing
        .run([$command, "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"));
    }
  };
}

mod it_shows_help_for {
  use super::*;

  help_test!(again);
  help_test!(archive);
  help_test!(budget);
  help_test!(cancel);
  help_test!(changes);
  help_test!(colors);
  help_test!(commands);
  help_test!(config);
  help_test!(done);
  help_test!(finish);
  help_test!(grep);
  help_test!(import);
  help_test!(last);
  help_test!(mark);
  help_test!(meanwhile);
  help_test!(note);
  help_test!(now);
  help_test!(on);
  help_test!(open);
  help_test!(recent);
  help_test!(reset);
  help_test!(rotate);
  help_test!(sections);
  help_test!(select);
  help_test!(show);
  help_test!(since);
  help_test!(tag);
  help_test!(tags);
  help_test!(template);
  help_test!(today);
  help_test!(undo);
  help_test!(view);
  help_test!(yesterday);
  help_test!(self_update, "self-update");
}
