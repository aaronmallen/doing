use super::Document;
use crate::{config::SortOrder, template::colors::STRIP_ANSI_RE};

/// Serialize a `Document` into the doing file format string.
///
/// Deduplicates entries by ID, sorts entries within each section according to
/// the given `sort_order`, and strips any ANSI color codes from the output.
pub fn serialize(doc: &Document, sort_order: SortOrder) -> String {
  let mut doc = doc.clone();
  doc.dedup();

  let mut out = String::new();

  for line in doc.other_content_top() {
    out.push_str(line);
    out.push('\n');
  }

  for (i, section) in doc.sections().iter().enumerate() {
    if i > 0 || !doc.other_content_top().is_empty() {
      out.push('\n');
    }

    out.push_str(section.title());
    out.push(':');

    let mut entries: Vec<_> = section.entries().to_vec();
    entries.sort_by(|a, b| a.date().cmp(&b.date()).then_with(|| a.title().cmp(b.title())));
    if sort_order == SortOrder::Desc {
      entries.reverse();
    }

    for entry in &entries {
      out.push_str(&format!("\n\t- {} | {}", entry.date().format("%Y-%m-%d %H:%M"), entry));
      if !entry.note().is_empty() {
        out.push_str(&format!("\n{}", entry.note()));
      }
    }
  }

  for line in doc.other_content_bottom() {
    out.push('\n');
    out.push_str(line);
  }

  strip_ansi(&out)
}

/// Remove ANSI escape sequences from a string.
fn strip_ansi(text: &str) -> String {
  STRIP_ANSI_RE.replace_all(text, "").into_owned()
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Entry, Note, Section, Tag, Tags};

  fn sample_date(hour: u32, minute: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2024, 3, 17, hour, minute, 0).unwrap()
  }

  mod serialize {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_produces_empty_string_for_empty_document() {
      let doc = Document::new();

      assert_eq!(serialize(&doc, SortOrder::Asc), "");
    }

    #[test]
    fn it_round_trips_a_well_formed_document() {
      let content = "\
Currently:
\t- 2024-03-17 14:30 | Working on feature @coding <aaaabbbbccccddddeeeeffffaaaabbbb>
\t\tA note about the work
Archive:
\t- 2024-03-16 10:00 | Old task @done(2024-03-16 11:00) <bbbbccccddddeeeeffffaaaabbbbcccc>";
      let doc = Document::parse(content);

      let output = serialize(&doc, SortOrder::Asc);

      assert_eq!(output, content);
    }

    #[test]
    fn it_sorts_entries_ascending() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        sample_date(15, 0),
        "Later task",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
      ));
      section.add_entry(Entry::new(
        sample_date(14, 0),
        "Earlier task",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
      ));
      doc.add_section(section);

      let output = serialize(&doc, SortOrder::Asc);

      let lines: Vec<&str> = output.lines().collect();
      assert!(lines[1].contains("Earlier task"));
      assert!(lines[2].contains("Later task"));
    }

    #[test]
    fn it_sorts_entries_descending() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        sample_date(14, 0),
        "Earlier task",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
      ));
      section.add_entry(Entry::new(
        sample_date(15, 0),
        "Later task",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
      ));
      doc.add_section(section);

      let output = serialize(&doc, SortOrder::Desc);

      let lines: Vec<&str> = output.lines().collect();
      assert!(lines[1].contains("Later task"));
      assert!(lines[2].contains("Earlier task"));
    }

    #[test]
    fn it_deduplicates_entries_by_id() {
      let mut doc = Document::new();
      let entry = Entry::new(
        sample_date(14, 30),
        "Task A",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      );
      let mut s1 = Section::new("Currently");
      s1.add_entry(entry.clone());
      let mut s2 = Section::new("Archive");
      s2.add_entry(entry);
      doc.add_section(s1);
      doc.add_section(s2);

      let output = serialize(&doc, SortOrder::Asc);

      assert_eq!(output.matches("Task A").count(), 1);
    }

    #[test]
    fn it_strips_ansi_color_codes() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        sample_date(14, 30),
        "\x1b[31mRed task\x1b[0m",
        Tags::new(),
        Note::new(),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      ));
      doc.add_section(section);

      let output = serialize(&doc, SortOrder::Asc);

      assert!(!output.contains("\x1b["));
      assert!(output.contains("Red task"));
    }

    #[test]
    fn it_preserves_other_content_top() {
      let mut doc = Document::new();
      doc.other_content_top_mut().push("# My Doing File".to_string());
      doc.add_section(Section::new("Currently"));

      let output = serialize(&doc, SortOrder::Asc);

      assert!(output.starts_with("# My Doing File\n"));
    }

    #[test]
    fn it_preserves_other_content_bottom() {
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      doc.other_content_bottom_mut().push("# Footer".to_string());

      let output = serialize(&doc, SortOrder::Asc);

      assert!(output.ends_with("# Footer"));
    }

    #[test]
    fn it_includes_notes() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        sample_date(14, 30),
        "Task with notes",
        Tags::new(),
        Note::from_str("A note line\nAnother note"),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      ));
      doc.add_section(section);

      let output = serialize(&doc, SortOrder::Asc);

      assert!(output.contains("\t\tA note line"));
      assert!(output.contains("\t\tAnother note"));
    }

    #[test]
    fn it_includes_tags() {
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        sample_date(14, 30),
        "Tagged task",
        Tags::from_iter(vec![
          Tag::new("coding", None::<String>),
          Tag::new("done", Some("2024-03-17 15:00")),
        ]),
        Note::new(),
        "Currently",
        Some("aaaabbbbccccddddeeeeffffaaaabbbb"),
      ));
      doc.add_section(section);

      let output = serialize(&doc, SortOrder::Asc);

      assert!(output.contains("@coding"));
      assert!(output.contains("@done(2024-03-17 15:00)"));
    }
  }

  mod strip_ansi {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_removes_ansi_escape_sequences() {
      let input = "\x1b[31mhello\x1b[0m world";

      assert_eq!(strip_ansi(input), "hello world");
    }

    #[test]
    fn it_returns_unchanged_string_without_ansi() {
      let input = "hello world";

      assert_eq!(strip_ansi(input), "hello world");
    }
  }
}
