use clap::Args;

use crate::{errors::Result, plugins};

/// List all registered export and import plugins.
///
/// Displays each plugin's name and trigger pattern, grouped by type
/// (Export / Import). Trigger patterns show what `--output` or `--type`
/// values each plugin responds to.
///
/// # Examples
///
/// ```text
/// doing plugins                    # list all registered plugins
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command;

impl Command {
  pub fn call(&self) -> Result<()> {
    println!("Export Plugins:");
    let export_registry = plugins::default_registry();
    for name in export_registry.available_formats() {
      if let Some(plugin) = export_registry.resolve(name) {
        let trigger = plugin.settings().trigger;
        println!("  {name} ({trigger})");
      }
    }

    println!();
    println!("Import Plugins:");
    let import_registry = plugins::import::default_registry();
    for name in import_registry.available_formats() {
      if let Some(plugin) = import_registry.resolve(name) {
        let trigger = plugin.settings().trigger;
        println!("  {name} ({trigger})");
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
      let cmd = Command;

      let result = cmd.call();

      assert!(result.is_ok());
    }
  }
}
