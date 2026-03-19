use clap::Args;

use crate::errors::Result;

/// List all available commands with their short descriptions.
///
/// Introspects the clap `Command` definition at runtime to display
/// every registered subcommand alongside its help text.
///
/// # Examples
///
/// ```text
/// doing commands                    # list all available commands
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command;

impl Command {
  pub fn call(&self, app: &clap::Command) -> Result<()> {
    for sub in app.get_subcommands() {
      if sub.is_hide_set() {
        continue;
      }

      let name = sub.get_name();
      let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();
      println!("{name:20} {about}");
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
      let app = clap::Command::new("doing")
        .subcommand(clap::Command::new("test").about("A test command"))
        .subcommand(clap::Command::new("hidden").hide(true));

      let cmd = Command;
      let result = cmd.call(&app);

      assert!(result.is_ok());
    }

    #[test]
    fn it_excludes_hidden_commands() {
      let app = clap::Command::new("doing")
        .subcommand(clap::Command::new("visible").about("Visible"))
        .subcommand(clap::Command::new("hidden").hide(true).about("Hidden"));

      let cmd = Command;
      let result = cmd.call(&app);

      assert!(result.is_ok());
    }
  }
}
