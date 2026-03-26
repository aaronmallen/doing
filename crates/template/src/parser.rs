use std::sync::LazyLock;

use regex::Regex;

use crate::colors;

static COLOR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"%((?:[fb]g?)?#[a-fA-F0-9]{6}|[a-zA-Z_]+)").unwrap());
static PLACEHOLDER_RE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(concat!(
    r"%(?P<width>-?\d+)?",
    r"(?:\^(?P<marker>.))?",
    r"(?:(?P<ichar>[ _t]|[^a-zA-Z0-9\s])(?P<icount>\d+))?",
    r"(?P<prefix>.[ _t]?)?",
    r"(?P<kind>shortdate|date|title|section|odnote|idnote|chompnote|note",
    r"|interval|duration|tags|hr_under|hr|n|t)\b",
  ))
  .unwrap()
});

const ESCAPE_SENTINEL: &str = "\u{E000}";

/// Indentation specification for wrapped/continuation lines.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Indent {
  pub count: u32,
  pub kind: IndentChar,
}

/// The type of indent character used for wrapped/continuation lines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndentChar {
  Custom(char),
  Space,
  Tab,
}

/// A parsed template element — either literal text, a color token, or a placeholder token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
  Color(colors::Color),
  Literal(String),
  Placeholder {
    indent: Option<Indent>,
    kind: TokenKind,
    marker: Option<char>,
    prefix: Option<String>,
    width: Option<i32>,
  },
}

/// The type of placeholder token in a template string.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenKind {
  Chompnote,
  Date,
  Duration,
  Hr,
  HrUnder,
  Idnote,
  Interval,
  Newline,
  Note,
  Odnote,
  Section,
  Shortdate,
  Tab,
  Tags,
  Title,
}

enum TokenMatch<'a> {
  Color {
    color: colors::Color,
    end: usize,
    start: usize,
  },
  Placeholder {
    caps: regex::Captures<'a>,
    end: usize,
    start: usize,
  },
}

impl TokenMatch<'_> {
  fn span(&self) -> (usize, usize) {
    match self {
      Self::Color {
        end,
        start,
        ..
      } => (*start, *end),
      Self::Placeholder {
        end,
        start,
        ..
      } => (*start, *end),
    }
  }
}

/// Parse a template string into a sequence of tokens.
///
/// Template strings contain literal text interspersed with `%` placeholder tokens.
/// Recognized tokens include `%date`, `%title`, `%note`, etc., with optional
/// width, alignment, marker, indent, and prefix modifiers. Color tokens like
/// `%cyan`, `%boldwhite`, `%reset`, and `%#FF5500` are also recognized.
///
/// Escaped percent signs (`\%`) become literal `%` in the output. Unrecognized
/// `%` sequences are preserved as literal text.
pub fn parse(template: &str) -> Vec<Token> {
  let escaped = template.replace("\\%", ESCAPE_SENTINEL);

  // Build a combined list of all matches sorted by position
  let mut matches: Vec<TokenMatch> = Vec::new();

  for caps in PLACEHOLDER_RE.captures_iter(&escaped) {
    let m = caps.get(0).unwrap();
    matches.push(TokenMatch::Placeholder {
      caps,
      end: m.end(),
      start: m.start(),
    });
  }

  for caps in COLOR_RE.captures_iter(&escaped) {
    let m = caps.get(0).unwrap();
    let color_str = caps.get(1).unwrap().as_str();
    if let Some((valid, orig_len)) = colors::validate_color(color_str) {
      // Only add if not overlapping with a placeholder match
      let start = m.start();
      let end = start + 1 + orig_len; // +1 for the % prefix
      let overlaps = matches.iter().any(|tm| {
        let (ts, te) = tm.span();
        start < te && end > ts
      });
      if !overlaps && let Some(color) = colors::Color::parse(&valid) {
        matches.push(TokenMatch::Color {
          color,
          end,
          start,
        });
      }
    }
  }

  matches.sort_by_key(|m| m.span().0);

  let mut tokens = Vec::new();
  let mut last_end = 0;

  for tm in &matches {
    let (start, end) = tm.span();

    if start > last_end {
      tokens.push(Token::Literal(unescape(&escaped[last_end..start])));
    }

    match tm {
      TokenMatch::Color {
        color, ..
      } => {
        tokens.push(Token::Color(color.clone()));
      }
      TokenMatch::Placeholder {
        caps, ..
      } => {
        let width = caps.name("width").map(|m| m.as_str().parse::<i32>().unwrap());
        let marker = caps.name("marker").and_then(|m| m.as_str().chars().next());

        let indent = caps.name("ichar").and_then(|ic| {
          caps.name("icount").map(|cnt| {
            let count = cnt.as_str().parse::<u32>().unwrap();
            let kind = match ic.as_str().chars().next().unwrap() {
              ' ' | '_' => IndentChar::Space,
              't' => IndentChar::Tab,
              c => IndentChar::Custom(c),
            };
            Indent {
              count,
              kind,
            }
          })
        });

        let prefix = caps.name("prefix").map(|m| m.as_str().to_string());

        let kind = match caps.name("kind").unwrap().as_str() {
          "chompnote" => TokenKind::Chompnote,
          "date" => TokenKind::Date,
          "duration" => TokenKind::Duration,
          "hr" => TokenKind::Hr,
          "hr_under" => TokenKind::HrUnder,
          "idnote" => TokenKind::Idnote,
          "interval" => TokenKind::Interval,
          "n" => TokenKind::Newline,
          "note" => TokenKind::Note,
          "odnote" => TokenKind::Odnote,
          "section" => TokenKind::Section,
          "shortdate" => TokenKind::Shortdate,
          "t" => TokenKind::Tab,
          "tags" => TokenKind::Tags,
          "title" => TokenKind::Title,
          _ => unreachable!(),
        };

        tokens.push(Token::Placeholder {
          indent,
          kind,
          marker,
          prefix,
          width,
        });
      }
    }

    last_end = end;
  }

  if last_end < escaped.len() {
    tokens.push(Token::Literal(unescape(&escaped[last_end..])));
  }

  tokens
}

fn unescape(s: &str) -> String {
  s.replace(ESCAPE_SENTINEL, "%")
}

#[cfg(test)]
mod test {
  use super::*;

  fn placeholder(kind: TokenKind) -> Token {
    Token::Placeholder {
      indent: None,
      kind,
      marker: None,
      prefix: None,
      width: None,
    }
  }

  mod parse {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_handles_escaped_percent() {
      let tokens = parse("\\%date is literal");

      assert_eq!(tokens, vec![Token::Literal("%date is literal".into())]);
    }

    #[test]
    fn it_parses_all_token_kinds() {
      for (input, expected) in [
        ("%chompnote", TokenKind::Chompnote),
        ("%date", TokenKind::Date),
        ("%duration", TokenKind::Duration),
        ("%hr", TokenKind::Hr),
        ("%hr_under", TokenKind::HrUnder),
        ("%idnote", TokenKind::Idnote),
        ("%interval", TokenKind::Interval),
        ("%n", TokenKind::Newline),
        ("%note", TokenKind::Note),
        ("%odnote", TokenKind::Odnote),
        ("%section", TokenKind::Section),
        ("%shortdate", TokenKind::Shortdate),
        ("%t", TokenKind::Tab),
        ("%tags", TokenKind::Tags),
        ("%title", TokenKind::Title),
      ] {
        let tokens = parse(input);

        assert_eq!(tokens.len(), 1, "expected one token for {input}");
        match &tokens[0] {
          Token::Placeholder {
            kind, ..
          } => {
            assert_eq!(*kind, expected, "wrong kind for {input}")
          }
          _ => panic!("expected placeholder for {input}"),
        }
      }
    }

    #[test]
    fn it_parses_color_tokens() {
      let tokens = parse("%cyan%date%reset");

      assert_eq!(
        tokens,
        vec![
          Token::Color(colors::Color::Named(colors::NamedColor::Cyan)),
          placeholder(TokenKind::Date),
          Token::Color(colors::Color::Named(colors::NamedColor::Reset)),
        ]
      );
    }

    #[test]
    fn it_parses_color_with_underscores() {
      let tokens = parse("%bold_white%title");

      assert_eq!(
        tokens,
        vec![
          Token::Color(colors::Color::Named(colors::NamedColor::BoldWhite)),
          placeholder(TokenKind::Title),
        ]
      );
    }

    #[test]
    fn it_parses_combined_width_indent_and_prefix() {
      let tokens = parse("%80_14\u{2503} note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: Some(Indent {
            count: 14,
            kind: IndentChar::Space,
          }),
          kind: TokenKind::Note,
          marker: None,
          prefix: Some("\u{2503} ".into()),
          width: Some(80),
        }]
      );
    }

    #[test]
    fn it_parses_empty_string() {
      let tokens = parse("");

      assert_eq!(tokens, vec![]);
    }

    #[test]
    fn it_parses_full_note_modifiers() {
      let tokens = parse("%^> 8: note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: Some(Indent {
            count: 8,
            kind: IndentChar::Space,
          }),
          kind: TokenKind::Note,
          marker: Some('>'),
          prefix: Some(": ".into()),
          width: None,
        }]
      );
    }

    #[test]
    fn it_parses_hex_color_tokens() {
      let tokens = parse("%#FF5500hello");

      assert_eq!(
        tokens,
        vec![
          Token::Color(colors::Color::Hex {
            background: false,
            b: 0x00,
            g: 0x55,
            r: 0xFF,
          }),
          Token::Literal("hello".into()),
        ]
      );
    }

    #[test]
    fn it_parses_literal_text() {
      let tokens = parse("hello world");

      assert_eq!(tokens, vec![Token::Literal("hello world".into())]);
    }

    #[test]
    fn it_parses_marker_modifier() {
      let tokens = parse("%^>note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: None,
          kind: TokenKind::Note,
          marker: Some('>'),
          prefix: None,
          width: None,
        }]
      );
    }

    #[test]
    fn it_parses_mixed_literals_and_placeholders() {
      let tokens = parse("hello %title world");

      assert_eq!(
        tokens,
        vec![
          Token::Literal("hello ".into()),
          placeholder(TokenKind::Title),
          Token::Literal(" world".into()),
        ]
      );
    }

    #[test]
    fn it_parses_negative_width_modifier() {
      let tokens = parse("%-10section");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: None,
          kind: TokenKind::Section,
          marker: None,
          prefix: None,
          width: Some(-10),
        }]
      );
    }

    #[test]
    fn it_parses_positive_width_modifier() {
      let tokens = parse("%80title");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: None,
          kind: TokenKind::Title,
          marker: None,
          prefix: None,
          width: Some(80),
        }]
      );
    }

    #[test]
    fn it_parses_prefix_modifier() {
      let tokens = parse("%: note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: None,
          kind: TokenKind::Note,
          marker: None,
          prefix: Some(": ".into()),
          width: None,
        }]
      );
    }

    #[test]
    fn it_parses_prefix_with_separator() {
      let tokens = parse("%80\u{2551} title");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: None,
          kind: TokenKind::Title,
          marker: None,
          prefix: Some("\u{2551} ".into()),
          width: Some(80),
        }]
      );
    }

    #[test]
    fn it_parses_space_indent_modifier() {
      let tokens = parse("% 4note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: Some(Indent {
            count: 4,
            kind: IndentChar::Space,
          }),
          kind: TokenKind::Note,
          marker: None,
          prefix: None,
          width: None,
        }]
      );
    }

    #[test]
    fn it_parses_tab_indent_modifier() {
      let tokens = parse("%t2note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: Some(Indent {
            count: 2,
            kind: IndentChar::Tab,
          }),
          kind: TokenKind::Note,
          marker: None,
          prefix: None,
          width: None,
        }]
      );
    }

    #[test]
    fn it_handles_control_characters_in_input() {
      // Entries containing old sentinel characters (\x01, \x02) should
      // render correctly without corruption now that we use PUA codepoints.
      let tokens = parse("hello \x01 and \x02 world");

      assert_eq!(tokens, vec![Token::Literal("hello \x01 and \x02 world".into())]);
    }

    #[test]
    fn it_parses_underscore_indent_modifier() {
      let tokens = parse("%_14note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: Some(Indent {
            count: 14,
            kind: IndentChar::Space,
          }),
          kind: TokenKind::Note,
          marker: None,
          prefix: None,
          width: None,
        }]
      );
    }

    #[test]
    fn it_preserves_unknown_percent_sequences() {
      let tokens = parse("%xyz%date");

      assert_eq!(
        tokens,
        vec![Token::Literal("%xyz".into()), placeholder(TokenKind::Date),]
      );
    }
  }
}
