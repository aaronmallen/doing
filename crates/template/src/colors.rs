use std::{
  fmt::{Display, Formatter, Result as FmtResult},
  sync::LazyLock,
};

use regex::Regex;
use yansi::{Condition, Style};

pub static STRIP_ANSI_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());

/// A named ANSI color or modifier that can appear as a `%color` token in templates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Color {
  Hex { b: u8, background: bool, g: u8, r: u8 },
  Named(NamedColor),
}

impl Color {
  /// Parse a color name string into a `Color`, if valid.
  ///
  /// Accepts named colors (`"cyan"`, `"boldwhite"`), modifiers (`"bold"`, `"italic"`),
  /// and hex RGB (`"#FF5500"`, `"bg#00FF00"`).
  pub fn parse(s: &str) -> Option<Self> {
    if let Some(hex) = parse_hex(s) {
      return Some(hex);
    }
    NamedColor::parse(s).map(Self::Named)
  }

  /// Return the ANSI escape sequence for this color.
  pub fn to_ansi(&self) -> String {
    match self {
      Self::Hex {
        background,
        b,
        g,
        r,
      } => {
        let style = if *background {
          Style::new().on_rgb(*r, *g, *b)
        } else {
          Style::new().rgb(*r, *g, *b)
        };
        style.prefix().to_string()
      }
      Self::Named(named) => named.to_ansi(),
    }
  }
}

impl Display for Color {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", self.to_ansi())
  }
}

/// A named ANSI color attribute.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NamedColor {
  Alert,
  BgBlack,
  BgBlue,
  BgCyan,
  BgGreen,
  BgMagenta,
  BgPurple,
  BgRed,
  BgWhite,
  BgYellow,
  Black,
  Blink,
  Blue,
  Bold,
  BoldBgBlack,
  BoldBgBlue,
  BoldBgCyan,
  BoldBgGreen,
  BoldBgMagenta,
  BoldBgPurple,
  BoldBgRed,
  BoldBgWhite,
  BoldBgYellow,
  BoldBlack,
  BoldBlue,
  BoldCyan,
  BoldGreen,
  BoldMagenta,
  BoldPurple,
  BoldRed,
  BoldWhite,
  BoldYellow,
  Chalkboard,
  Concealed,
  Cyan,
  Dark,
  Default,
  Error,
  Flamingo,
  Green,
  Hotpants,
  Italic,
  Knightrider,
  Led,
  Magenta,
  Negative,
  Purple,
  RapidBlink,
  Red,
  Redacted,
  Reset,
  Softpurple,
  Strike,
  Strikethrough,
  Underline,
  Underscore,
  White,
  Whiteboard,
  Yeller,
  Yellow,
}

impl NamedColor {
  /// Parse a string into a `NamedColor`, if it matches a known color name.
  pub fn parse(s: &str) -> Option<Self> {
    let normalized = s.replace('_', "").replace("bright", "bold");
    let normalized = if normalized.starts_with("bgbold") {
      normalized.replacen("bgbold", "boldbg", 1)
    } else {
      normalized
    };
    match normalized.as_str() {
      "alert" => Some(Self::Alert),
      "bgblack" => Some(Self::BgBlack),
      "bgblue" => Some(Self::BgBlue),
      "bgcyan" => Some(Self::BgCyan),
      "bggreen" => Some(Self::BgGreen),
      "bgmagenta" | "bgpurple" => Some(Self::BgMagenta),
      "bgred" => Some(Self::BgRed),
      "bgwhite" => Some(Self::BgWhite),
      "bgyellow" => Some(Self::BgYellow),
      "black" => Some(Self::Black),
      "blink" => Some(Self::Blink),
      "blue" => Some(Self::Blue),
      "bold" => Some(Self::Bold),
      "boldbgblack" => Some(Self::BoldBgBlack),
      "boldbgblue" => Some(Self::BoldBgBlue),
      "boldbgcyan" => Some(Self::BoldBgCyan),
      "boldbggreen" => Some(Self::BoldBgGreen),
      "boldbgmagenta" | "boldbgpurple" => Some(Self::BoldBgMagenta),
      "boldbgred" => Some(Self::BoldBgRed),
      "boldbgwhite" => Some(Self::BoldBgWhite),
      "boldbgyellow" => Some(Self::BoldBgYellow),
      "boldblack" => Some(Self::BoldBlack),
      "boldblue" => Some(Self::BoldBlue),
      "boldcyan" => Some(Self::BoldCyan),
      "boldgreen" => Some(Self::BoldGreen),
      "boldmagenta" | "boldpurple" => Some(Self::BoldMagenta),
      "boldred" => Some(Self::BoldRed),
      "boldwhite" => Some(Self::BoldWhite),
      "boldyellow" => Some(Self::BoldYellow),
      "chalkboard" => Some(Self::Chalkboard),
      "clear" | "reset" => Some(Self::Reset),
      "concealed" => Some(Self::Concealed),
      "cyan" => Some(Self::Cyan),
      "dark" => Some(Self::Dark),
      "default" => Some(Self::Default),
      "error" => Some(Self::Error),
      "flamingo" => Some(Self::Flamingo),
      "green" => Some(Self::Green),
      "hotpants" => Some(Self::Hotpants),
      "italic" => Some(Self::Italic),
      "knightrider" => Some(Self::Knightrider),
      "led" => Some(Self::Led),
      "magenta" | "purple" => Some(Self::Magenta),
      "negative" => Some(Self::Negative),
      "rapidblink" => Some(Self::RapidBlink),
      "red" => Some(Self::Red),
      "redacted" => Some(Self::Redacted),
      "softpurple" => Some(Self::Softpurple),
      "strike" => Some(Self::Strike),
      "strikethrough" => Some(Self::Strikethrough),
      "underline" | "underscore" => Some(Self::Underline),
      "white" => Some(Self::White),
      "whiteboard" => Some(Self::Whiteboard),
      "yeller" => Some(Self::Yeller),
      "yellow" => Some(Self::Yellow),
      _ => None,
    }
  }

  /// Return the ANSI escape sequence for this named color.
  fn to_ansi(self) -> String {
    if !yansi::is_enabled() {
      return String::new();
    }
    // Reset and default need raw ANSI since yansi styles compose forward
    match self {
      Self::Default => "\x1b[0;39m".into(),
      Self::Reset => "\x1b[0m".into(),
      _ => self.to_style().prefix().to_string(),
    }
  }

  /// Convert this named color into a [`yansi::Style`].
  ///
  /// For most colors, this maps directly to yansi's `Color` and `Attribute` types.
  /// Compound themes (e.g. `chalkboard`, `flamingo`) compose multiple style properties.
  fn to_style(self) -> Style {
    match self {
      Self::Alert => Style::new().red().on_yellow().bold(),
      Self::BgBlack => Style::new().on_black(),
      Self::BgBlue => Style::new().on_blue(),
      Self::BgCyan => Style::new().on_cyan(),
      Self::BgGreen => Style::new().on_green(),
      Self::BgMagenta | Self::BgPurple => Style::new().on_magenta(),
      Self::BgRed => Style::new().on_red(),
      Self::BgWhite => Style::new().on_white(),
      Self::BgYellow => Style::new().on_yellow(),
      Self::Black => Style::new().black(),
      Self::Blink => Style::new().blink(),
      Self::Blue => Style::new().blue(),
      Self::Bold => Style::new().bold(),
      Self::BoldBgBlack => Style::new().on_bright_black(),
      Self::BoldBgBlue => Style::new().on_bright_blue(),
      Self::BoldBgCyan => Style::new().on_bright_cyan(),
      Self::BoldBgGreen => Style::new().on_bright_green(),
      Self::BoldBgMagenta | Self::BoldBgPurple => Style::new().on_bright_magenta(),
      Self::BoldBgRed => Style::new().on_bright_red(),
      Self::BoldBgWhite => Style::new().on_bright_white(),
      Self::BoldBgYellow => Style::new().on_bright_yellow(),
      Self::BoldBlack => Style::new().bright_black(),
      Self::BoldBlue => Style::new().bright_blue(),
      Self::BoldCyan => Style::new().bright_cyan(),
      Self::BoldGreen => Style::new().bright_green(),
      Self::BoldMagenta | Self::BoldPurple => Style::new().bright_magenta(),
      Self::BoldRed => Style::new().bright_red(),
      Self::BoldWhite => Style::new().bright_white(),
      Self::BoldYellow => Style::new().bright_yellow(),
      Self::Chalkboard => Style::new().white().on_black().bold(),
      Self::Concealed => Style::new().conceal(),
      Self::Cyan => Style::new().cyan(),
      Self::Dark => Style::new().dim(),
      Self::Default | Self::Reset => Style::new(),
      Self::Error => Style::new().white().on_red().bold(),
      Self::Flamingo => Style::new().red().on_white().invert(),
      Self::Green => Style::new().green(),
      Self::Hotpants => Style::new().blue().on_black().invert(),
      Self::Italic => Style::new().italic(),
      Self::Knightrider => Style::new().black().on_black().invert(),
      Self::Led => Style::new().green().on_black(),
      Self::Magenta | Self::Purple => Style::new().magenta(),
      Self::Negative => Style::new().invert(),
      Self::RapidBlink => Style::new().rapid_blink(),
      Self::Red => Style::new().red(),
      Self::Redacted => Style::new().black().on_black(),
      Self::Softpurple => Style::new().magenta().on_black(),
      Self::Strike | Self::Strikethrough => Style::new().strike(),
      Self::Underline | Self::Underscore => Style::new().underline(),
      Self::White => Style::new().white(),
      Self::Whiteboard => Style::new().black().on_white().bold(),
      Self::Yeller => Style::new().white().on_yellow().bold(),
      Self::Yellow => Style::new().yellow(),
    }
  }
}

impl Display for NamedColor {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", self.to_ansi())
  }
}

/// Initialize color output with terminal detection.
///
/// Call this once at program startup to enable automatic TTY and `NO_COLOR` / `CLICOLOR`
/// detection via [`yansi`]. When output is piped or redirected, colors are suppressed.
pub fn init() {
  yansi::whenever(Condition::TTY_AND_COLOR);
}

/// Strip all ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
  STRIP_ANSI_RE.replace_all(s, "").into_owned()
}

/// Validate a color name string, returning the longest valid prefix.
///
/// This allows color tokens to bleed into adjacent text, e.g. `%greensomething`
/// still matches `green`.
pub fn validate_color(s: &str) -> Option<String> {
  let normalized = s.replace('_', "").replace("bright", "bold");
  let normalized = if normalized.starts_with("bgbold") {
    normalized.replacen("bgbold", "boldbg", 1)
  } else {
    normalized
  };

  // Check for hex color
  if let Some(rest) = normalized.strip_prefix('#')
    && rest.len() >= 6
    && rest[..6].chars().all(|c| c.is_ascii_hexdigit())
  {
    return Some(format!("#{}", &rest[..6]));
  }
  for prefix in ["fg#", "bg#", "f#", "b#"] {
    if let Some(rest) = normalized.strip_prefix(prefix)
      && rest.len() >= 6
      && rest[..6].chars().all(|c| c.is_ascii_hexdigit())
    {
      return Some(format!("{prefix}{}", &rest[..6]));
    }
  }

  // Find longest matching named color
  let mut valid = None;
  let mut compiled = String::new();
  for ch in normalized.chars() {
    compiled.push(ch);
    if NamedColor::parse(&compiled).is_some() {
      valid = Some(compiled.clone());
    }
  }
  valid
}

/// Return the visible (non-ANSI) display width of a string.
///
/// Uses Unicode display widths so CJK characters count as 2 and emoji count as 2.
pub fn visible_len(s: &str) -> usize {
  unicode_width::UnicodeWidthStr::width(strip_ansi(s).as_str())
}

/// Return a sorted list of all supported color names.
#[cfg(test)]
fn available_colors() -> Vec<&'static str> {
  vec![
    "alert",
    "bgblack",
    "bgblue",
    "bgcyan",
    "bggreen",
    "bgmagenta",
    "bgpurple",
    "bgred",
    "bgwhite",
    "bgyellow",
    "black",
    "blink",
    "blue",
    "bold",
    "boldbgblack",
    "boldbgblue",
    "boldbgcyan",
    "boldbggreen",
    "boldbgmagenta",
    "boldbgpurple",
    "boldbgred",
    "boldbgwhite",
    "boldbgyellow",
    "boldblack",
    "boldblue",
    "boldcyan",
    "boldgreen",
    "boldmagenta",
    "boldpurple",
    "boldred",
    "boldwhite",
    "boldyellow",
    "chalkboard",
    "clear",
    "concealed",
    "cyan",
    "dark",
    "default",
    "error",
    "flamingo",
    "green",
    "hotpants",
    "italic",
    "knightrider",
    "led",
    "magenta",
    "negative",
    "purple",
    "rapidblink",
    "red",
    "redacted",
    "reset",
    "softpurple",
    "strike",
    "strikethrough",
    "underline",
    "underscore",
    "white",
    "whiteboard",
    "yeller",
    "yellow",
  ]
}

fn parse_hex(s: &str) -> Option<Color> {
  let (background, hex_str) = if let Some(rest) = s.strip_prefix("bg#").or_else(|| s.strip_prefix("b#")) {
    (true, rest)
  } else if let Some(rest) = s.strip_prefix("fg#").or_else(|| s.strip_prefix("f#")) {
    (false, rest)
  } else if let Some(rest) = s.strip_prefix('#') {
    (false, rest)
  } else {
    return None;
  };

  if hex_str.len() != 6 || !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
    return None;
  }

  let r = u8::from_str_radix(&hex_str[0..2], 16).ok()?;
  let g = u8::from_str_radix(&hex_str[2..4], 16).ok()?;
  let b = u8::from_str_radix(&hex_str[4..6], 16).ok()?;

  Some(Color::Hex {
    background,
    b,
    g,
    r,
  })
}

#[cfg(test)]
mod test {
  use super::*;

  mod available_colors {
    use super::*;

    #[test]
    fn it_contains_basic_colors() {
      let colors = available_colors();

      assert!(colors.contains(&"red"));
      assert!(colors.contains(&"green"));
      assert!(colors.contains(&"blue"));
      assert!(colors.contains(&"cyan"));
      assert!(colors.contains(&"yellow"));
      assert!(colors.contains(&"magenta"));
      assert!(colors.contains(&"white"));
      assert!(colors.contains(&"black"));
    }

    #[test]
    fn it_contains_reset() {
      let colors = available_colors();

      assert!(colors.contains(&"reset"));
      assert!(colors.contains(&"default"));
    }

    #[test]
    fn it_returns_sorted_list() {
      let colors = available_colors();

      let mut sorted = colors.clone();
      sorted.sort();
      assert_eq!(colors, sorted);
    }
  }

  mod color_parse {
    use super::*;

    #[test]
    fn it_normalizes_bright_to_bold() {
      let color = Color::parse("brightwhite");

      assert_eq!(color, Some(Color::Named(NamedColor::BoldWhite)));
    }

    #[test]
    fn it_normalizes_underscores() {
      let color = Color::parse("bold_white");

      assert_eq!(color, Some(Color::Named(NamedColor::BoldWhite)));
    }

    #[test]
    fn it_parses_bold_color() {
      let color = Color::parse("boldwhite");

      assert_eq!(color, Some(Color::Named(NamedColor::BoldWhite)));
    }

    #[test]
    fn it_parses_hex_background() {
      let color = Color::parse("bg#00FF00");

      assert_eq!(
        color,
        Some(Color::Hex {
          background: true,
          b: 0x00,
          g: 0xFF,
          r: 0x00,
        })
      );
    }

    #[test]
    fn it_parses_hex_foreground() {
      let color = Color::parse("#FF5500");

      assert_eq!(
        color,
        Some(Color::Hex {
          background: false,
          b: 0x00,
          g: 0x55,
          r: 0xFF,
        })
      );
    }

    #[test]
    fn it_parses_named_color() {
      let color = Color::parse("cyan");

      assert_eq!(color, Some(Color::Named(NamedColor::Cyan)));
    }

    #[test]
    fn it_returns_none_for_invalid() {
      assert_eq!(Color::parse("notacolor"), None);
    }
  }

  mod color_to_ansi {
    use super::*;

    #[test]
    fn it_emits_empty_when_disabled() {
      yansi::disable();

      let result = Color::Named(NamedColor::Cyan).to_ansi();

      assert_eq!(result, "");
      yansi::enable();
    }

    #[test]
    fn it_emits_hex_background() {
      yansi::enable();
      let color = Color::Hex {
        background: true,
        b: 0x00,
        g: 0xFF,
        r: 0x00,
      };

      let result = color.to_ansi();

      assert!(result.contains("48;2;0;255;0"), "expected RGB bg escape, got: {result}");
    }

    #[test]
    fn it_emits_hex_foreground() {
      yansi::enable();
      let color = Color::Hex {
        background: false,
        b: 0x00,
        g: 0x55,
        r: 0xFF,
      };

      let result = color.to_ansi();

      assert!(
        result.contains("38;2;255;85;0"),
        "expected RGB fg escape, got: {result}"
      );
    }

    #[test]
    fn it_emits_named_ansi() {
      yansi::enable();

      let result = Color::Named(NamedColor::Cyan).to_ansi();

      assert!(result.contains("36"), "expected cyan code 36, got: {result}");
    }

    #[test]
    fn it_emits_reset() {
      yansi::enable();

      let result = Color::Named(NamedColor::Reset).to_ansi();

      assert_eq!(result, "\x1b[0m");
    }
  }

  mod strip_ansi {
    use pretty_assertions::assert_eq;

    use super::super::strip_ansi;

    #[test]
    fn it_removes_escape_sequences() {
      let input = "\x1b[36mhello\x1b[0m world";

      assert_eq!(strip_ansi(input), "hello world");
    }

    #[test]
    fn it_returns_plain_text_unchanged() {
      assert_eq!(strip_ansi("hello"), "hello");
    }
  }

  mod validate_color {
    use pretty_assertions::assert_eq;

    use super::super::validate_color;

    #[test]
    fn it_finds_longest_prefix() {
      assert_eq!(validate_color("boldbluefoo"), Some("boldblue".into()));
    }

    #[test]
    fn it_returns_none_for_invalid() {
      assert_eq!(validate_color("notacolor"), None);
    }

    #[test]
    fn it_validates_bg_hex() {
      assert_eq!(validate_color("bg#00FF00"), Some("bg#00FF00".into()));
    }

    #[test]
    fn it_validates_bold_color() {
      assert_eq!(validate_color("boldwhite"), Some("boldwhite".into()));
    }

    #[test]
    fn it_validates_hex() {
      assert_eq!(validate_color("#FF5500"), Some("#FF5500".into()));
    }

    #[test]
    fn it_validates_simple_color() {
      assert_eq!(validate_color("cyan"), Some("cyan".into()));
    }
  }

  mod visible_len {
    use pretty_assertions::assert_eq;

    use super::super::visible_len;

    #[test]
    fn it_counts_plain_text() {
      assert_eq!(visible_len("hello"), 5);
    }

    #[test]
    fn it_counts_cjk_characters_as_double_width() {
      // CJK characters are 2 display columns each
      assert_eq!(visible_len("日本語"), 6);
    }

    #[test]
    fn it_counts_emoji_as_double_width() {
      // Most emoji are 2 display columns
      assert_eq!(visible_len("🎉"), 2);
    }

    #[test]
    fn it_excludes_ansi_codes() {
      let input = "\x1b[36mhello\x1b[0m";

      assert_eq!(visible_len(input), 5);
    }
  }
}
