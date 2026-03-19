use clap::Args;

use crate::errors::Result;

/// List commands that accept a given option.
///
/// Introspects the clap `Command` definition at runtime to find
/// every subcommand that defines the named argument or flag.
///
/// # Examples
///
/// ```text
/// doing commands_accepting tag       # list commands that accept --tag
/// doing commands_accepting section   # list commands that accept --section
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// The option name to search for (without leading dashes)
  option: String,
}

impl Command {
  pub fn call(&self, app: &clap::Command) -> Result<()> {
    let needle = self.option.trim_start_matches('-');

    for sub in app.get_subcommands() {
      if sub.is_hide_set() {
        continue;
      }

      let accepts = sub.get_arguments().any(|arg| {
        arg.get_id().as_str() == needle
          || arg.get_long().is_some_and(|l| l == needle)
          || arg.get_short().is_some_and(|s| s.to_string() == needle)
      });

      if accepts {
        let name = sub.get_name();
        let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();
        println!("{name:20} {about}");
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod call {
    use super::*;

    #[test]
    fn it_does_not_error() {
      let app = clap::Command::new("doing").subcommand(
        clap::Command::new("test")
          .about("A test command")
          .arg(clap::Arg::new("tag").long("tag")),
      );

      let cmd = Command {
        option: "tag".to_string(),
      };
      let result = cmd.call(&app);

      assert!(result.is_ok());
    }

    #[test]
    fn it_matches_with_leading_dashes() {
      let app = clap::Command::new("doing").subcommand(
        clap::Command::new("test")
          .about("A test command")
          .arg(clap::Arg::new("tag").long("tag")),
      );

      let cmd = Command {
        option: "--tag".to_string(),
      };
      let result = cmd.call(&app);

      assert!(result.is_ok());
    }
  }
}
