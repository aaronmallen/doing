use clap::Args;
use doing_plugins as plugins;

use crate::Result;

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
pub struct Command {
  /// Display plugins in column format
  #[arg(short, long)]
  column: bool,

  /// Filter plugins by type (import/export)
  #[arg(short = 't', long = "type")]
  plugin_type: Option<String>,
}

impl Command {
  pub fn call(&self) -> Result<()> {
    if let Some(ref t) = self.plugin_type
      && !t.eq_ignore_ascii_case("export")
      && !t.eq_ignore_ascii_case("import")
    {
      return Err(crate::Error::Config(format!(
        "invalid plugin type '{t}'. Valid types: import, export"
      )));
    }

    let show_export = self
      .plugin_type
      .as_ref()
      .map(|t| t.eq_ignore_ascii_case("export"))
      .unwrap_or(true);
    let show_import = self
      .plugin_type
      .as_ref()
      .map(|t| t.eq_ignore_ascii_case("import"))
      .unwrap_or(true);

    if show_export {
      if !self.column {
        println!("Export Plugins:");
      }
      let export_registry = plugins::default_registry()?;
      for name in export_registry.available_formats() {
        if let Some(plugin) = export_registry.resolve(name) {
          if self.column {
            println!("{name}");
          } else {
            let trigger = plugin.settings().trigger;
            println!("  {name} ({trigger})");
          }
        }
      }
    }

    if show_export && show_import && !self.column {
      println!();
    }

    if show_import {
      if !self.column {
        println!("Import Plugins:");
      }
      let import_registry = plugins::import::default_registry()?;
      for name in import_registry.available_formats() {
        if let Some(plugin) = import_registry.resolve(name) {
          if self.column {
            println!("{name}");
          } else {
            let trigger = plugin.settings().trigger;
            println!("  {name} ({trigger})");
          }
        }
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
      let cmd = Command {
        column: false,
        plugin_type: None,
      };

      let result = cmd.call();

      assert!(result.is_ok());
    }

    #[test]
    fn it_errors_on_invalid_type() {
      let cmd = Command {
        column: false,
        plugin_type: Some("foo".into()),
      };

      let result = cmd.call();

      assert!(result.is_err());
      let err = result.unwrap_err().to_string();
      assert!(err.contains("invalid plugin type"), "unexpected error: {err}");
    }
  }
}
