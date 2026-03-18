use regex::Regex;

const ESCAPE_SENTINEL: &str = "\x01";

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

/// A parsed template element — either literal text or a placeholder token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
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

/// Parse a template string into a sequence of tokens.
///
/// Template strings contain literal text interspersed with `%` placeholder tokens.
/// Recognized tokens include `%date`, `%title`, `%note`, etc., with optional
/// width, alignment, marker, indent, and prefix modifiers.
///
/// Escaped percent signs (`\%`) become literal `%` in the output. Unrecognized
/// `%` sequences are preserved as literal text.
pub fn parse(template: &str) -> Vec<Token> {
  let escaped = template.replace("\\%", ESCAPE_SENTINEL);

  let re = Regex::new(concat!(
    r"%(?P<width>-?\d+)?",
    r"(?:\^(?P<marker>.))?",
    r"(?:(?P<ichar>[ _t]|[^a-zA-Z0-9\s])(?P<icount>\d+))?",
    r"(?P<prefix>.[ _t]?)?",
    r"(?P<kind>shortdate|date|title|section|odnote|idnote|chompnote|note",
    r"|interval|duration|tags|hr_under|hr|n|t)\b",
  ))
  .expect("template token regex is valid");

  let mut tokens = Vec::new();
  let mut last_end = 0;

  for caps in re.captures_iter(&escaped) {
    let m = caps.get(0).unwrap();

    if m.start() > last_end {
      tokens.push(Token::Literal(unescape(&escaped[last_end..m.start()])));
    }

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

    last_end = m.end();
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
          } => assert_eq!(*kind, expected, "wrong kind for {input}"),
          _ => panic!("expected placeholder for {input}"),
        }
      }
    }

    #[test]
    fn it_parses_combined_width_indent_and_prefix() {
      let tokens = parse("%80_14┃ note");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: Some(Indent {
            count: 14,
            kind: IndentChar::Space,
          }),
          kind: TokenKind::Note,
          marker: None,
          prefix: Some("┃ ".into()),
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
      let tokens = parse("%80║ title");

      assert_eq!(
        tokens,
        vec![Token::Placeholder {
          indent: None,
          kind: TokenKind::Title,
          marker: None,
          prefix: Some("║ ".into()),
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
      let tokens = parse("%cyan%date%reset");

      assert_eq!(
        tokens,
        vec![
          Token::Literal("%cyan".into()),
          placeholder(TokenKind::Date),
          Token::Literal("%reset".into()),
        ]
      );
    }
  }
}
