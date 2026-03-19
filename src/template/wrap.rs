use regex::Regex;

use super::colors;

const TAG_VALUE_SENTINEL: &str = "\x02\x02\x02\x02";

/// Wrap text at word boundaries, respecting tag values.
///
/// Tag values like `@tag(value with spaces)` are treated as single units
/// and will not be broken across lines. Width is measured by visible
/// characters (ANSI escapes are excluded from the count).
///
/// Returns the original text unchanged if `width` is 0.
pub fn wrap(text: &str, width: usize) -> String {
  if width == 0 || text.is_empty() {
    return text.to_string();
  }

  let tag_re = Regex::new(r"@\S+\(.*?\)").expect("tag value regex is valid");

  // Protect spaces inside tag values from splitting
  let protected = tag_re.replace_all(text, |caps: &regex::Captures| caps[0].replace(' ', TAG_VALUE_SENTINEL));

  let normalized = protected.replace('\n', " ");
  let words: Vec<String> = normalized
    .split(' ')
    .map(|w| w.replace(TAG_VALUE_SENTINEL, " "))
    .collect();

  let mut lines: Vec<String> = Vec::new();
  let mut current_line: Vec<String> = Vec::new();

  for word in words {
    let word_len = colors::visible_len(&word);

    if word_len >= width {
      // Flush current line
      if !current_line.is_empty() {
        lines.push(current_line.join(" "));
        current_line.clear();
      }
      // Keep tag values atomic (don't break @tag(value) across lines)
      if word.starts_with('@') && word.contains('(') {
        current_line.push(word);
        continue;
      }
      // Break other long words into chunks
      let visible: String = colors::strip_ansi(&word);
      let mut chars = visible.chars().peekable();
      while chars.peek().is_some() {
        let chunk: String = chars.by_ref().take(width).collect();
        if chars.peek().is_some() {
          lines.push(chunk);
        } else {
          current_line.push(chunk);
        }
      }
      continue;
    }

    let current_len = colors::visible_len(&current_line.join(" "));
    if !current_line.is_empty() && current_len + word_len + 1 > width {
      lines.push(current_line.join(" "));
      current_line.clear();
    }

    current_line.push(word);
  }

  if !current_line.is_empty() {
    lines.push(current_line.join(" "));
  }

  lines.join("\n")
}

/// Wrap text with indentation applied to continuation lines.
///
/// The first line wraps at `width`. Subsequent lines are prefixed with
/// `indent` characters of whitespace and wrap at `width - indent` to
/// stay within the total width.
///
/// Returns the original text unchanged if `width` is 0.
pub fn wrap_with_indent(text: &str, width: usize, indent: usize) -> String {
  if width == 0 || text.is_empty() {
    return text.to_string();
  }

  let continuation_width = width.saturating_sub(indent);
  if continuation_width == 0 {
    return wrap(text, width);
  }

  let tag_re = Regex::new(r"@\S+\(.*?\)").expect("tag value regex is valid");
  let protected = tag_re.replace_all(text, |caps: &regex::Captures| caps[0].replace(' ', TAG_VALUE_SENTINEL));
  let normalized = protected.replace('\n', " ");
  let words: Vec<String> = normalized
    .split(' ')
    .map(|w| w.replace(TAG_VALUE_SENTINEL, " "))
    .collect();

  let indent_str: String = " ".repeat(indent);
  let mut lines: Vec<String> = Vec::new();
  let mut current_line: Vec<String> = Vec::new();
  let mut is_first_line = true;

  for word in words {
    let word_len = colors::visible_len(&word);
    let effective_width = if is_first_line { width } else { continuation_width };

    let current_len = colors::visible_len(&current_line.join(" "));
    if !current_line.is_empty() && current_len + word_len + 1 > effective_width {
      let line = current_line.join(" ");
      if is_first_line {
        lines.push(line);
        is_first_line = false;
      } else {
        lines.push(format!("{indent_str}{line}"));
      }
      current_line.clear();
    }

    current_line.push(word);
  }

  if !current_line.is_empty() {
    let line = current_line.join(" ");
    if is_first_line {
      lines.push(line);
    } else {
      lines.push(format!("{indent_str}{line}"));
    }
  }

  lines.join("\n")
}

#[cfg(test)]
mod test {
  mod wrap {
    use pretty_assertions::assert_eq;

    use super::super::wrap;

    #[test]
    fn it_breaks_long_words() {
      let result = wrap("abcdefghij", 4);

      assert_eq!(result, "abcd\nefgh\nij");
    }

    #[test]
    fn it_preserves_tag_values() {
      let result = wrap("hello @tag(value with spaces) world", 20);

      assert_eq!(result, "hello\n@tag(value with spaces)\nworld");
    }

    #[test]
    fn it_returns_empty_for_empty_input() {
      assert_eq!(wrap("", 40), "");
    }

    #[test]
    fn it_returns_unchanged_when_width_is_zero() {
      assert_eq!(wrap("hello world", 0), "hello world");
    }

    #[test]
    fn it_returns_unchanged_when_within_width() {
      assert_eq!(wrap("hello world", 40), "hello world");
    }

    #[test]
    fn it_wraps_at_word_boundaries() {
      let result = wrap("the quick brown fox jumps over", 16);

      assert_eq!(result, "the quick brown\nfox jumps over");
    }

    #[test]
    fn it_wraps_multiple_lines() {
      let result = wrap("one two three four five six", 10);

      assert_eq!(result, "one two\nthree four\nfive six");
    }
  }

  mod wrap_with_indent {
    use pretty_assertions::assert_eq;

    use super::super::wrap_with_indent;

    #[test]
    fn it_does_not_indent_first_line() {
      let result = wrap_with_indent("hello world foo bar", 12, 4);

      assert_eq!(result, "hello world\n    foo bar");
    }

    #[test]
    fn it_indents_continuation_lines() {
      let result = wrap_with_indent("one two three four", 10, 2);

      assert_eq!(result, "one two\n  three\n  four");
    }

    #[test]
    fn it_returns_unchanged_when_width_is_zero() {
      assert_eq!(wrap_with_indent("hello world", 0, 4), "hello world");
    }
  }
}
