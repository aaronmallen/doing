use clap::Args;
use doing_template::colors;

use crate::Result;

const BACKGROUND_COLORS: [&str; 9] = [
  "bgblack",
  "bgblue",
  "bgcyan",
  "bggreen",
  "bgmagenta",
  "bgpurple",
  "bgred",
  "bgwhite",
  "bgyellow",
];

const BOLD_BG_COLORS: [&str; 9] = [
  "boldbgblack",
  "boldbgblue",
  "boldbgcyan",
  "boldbggreen",
  "boldbgmagenta",
  "boldbgpurple",
  "boldbgred",
  "boldbgwhite",
  "boldbgyellow",
];

const BOLD_COLORS: [&str; 10] = [
  "boldblack",
  "boldblue",
  "boldcyan",
  "boldgreen",
  "boldmagenta",
  "boldpurple",
  "boldred",
  "boldwhite",
  "boldyellow",
  "dark",
];

const FOREGROUND_COLORS: [&str; 9] = [
  "black", "blue", "cyan", "green", "magenta", "purple", "red", "white", "yellow",
];

const MODIFIERS: [&str; 10] = [
  "blink",
  "bold",
  "concealed",
  "italic",
  "negative",
  "rapidblink",
  "strike",
  "strikethrough",
  "underline",
  "underscore",
];

const RESETS: [&str; 3] = ["clear", "default", "reset"];

const SWATCH: &str = "    ";

const THEMES: [&str; 11] = [
  "alert",
  "chalkboard",
  "error",
  "flamingo",
  "hotpants",
  "knightrider",
  "led",
  "redacted",
  "softpurple",
  "whiteboard",
  "yeller",
];

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
    let reset = colors::Color::parse("reset").map(|c| c.to_ansi()).unwrap_or_default();

    print_section("Foreground Colors", &FOREGROUND_COLORS, &reset);
    print_section("Bold/Bright Colors", &BOLD_COLORS, &reset);
    print_section("Background Colors", &BACKGROUND_COLORS, &reset);
    print_section("Bold/Bright Backgrounds", &BOLD_BG_COLORS, &reset);
    print_section("Modifiers", &MODIFIERS, &reset);
    print_section("Themes", &THEMES, &reset);
    print_section("Reset/Default", &RESETS, &reset);

    println!("Hex RGB syntax:");
    println!("  %#FF5500      foreground color");
    println!("  %fg#FF5500    foreground color (explicit)");
    println!("  %bg#00FF00    background color");

    Ok(())
  }
}

fn print_section(header: &str, names: &[&str], reset: &str) {
  println!("{header}:");
  for name in names {
    if let Some(color) = colors::Color::parse(name) {
      let ansi = color.to_ansi();
      println!("  {ansi}{SWATCH}{reset}  %{name}");
    }
  }
  println!();
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

  mod print_section {
    use super::*;

    #[test]
    fn it_prints_header_and_entries() {
      let reset = colors::Color::parse("reset").map(|c| c.to_ansi()).unwrap_or_default();

      // Should not panic
      print_section("Test Section", &["red", "blue"], &reset);
    }
  }
}
