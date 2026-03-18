use clap::Args;

use crate::{errors::Result, template::colors};

/// Show all available color template tokens with visual swatches.
///
/// Lists named colors, bold/background variants, modifiers, and themes.
/// Each color is displayed with a swatch showing how it renders in the terminal.
/// Hex RGB syntax is also documented for custom colors.
///
/// # Examples
///
/// ```text
/// doing colors                    # list all colors with swatches
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command;

impl Command {
  pub fn call(&self) -> Result<()> {
    let names = colors::available_colors();
    let reset = colors::Color::parse("reset").map(|c| c.to_ansi()).unwrap_or_default();

    for name in &names {
      if let Some(color) = colors::Color::parse(name) {
        let ansi = color.to_ansi();
        println!("{ansi}%{name}{reset}");
      }
    }

    println!();
    println!("Hex RGB syntax:");
    println!("  %#FF5500      foreground color");
    println!("  %fg#FF5500    foreground color (explicit)");
    println!("  %bg#00FF00    background color");

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  mod call {
    use super::*;

    #[test]
    fn it_runs_without_error() {
      let cmd = Command;

      let result = cmd.call();

      assert!(result.is_ok());
    }
  }
}
