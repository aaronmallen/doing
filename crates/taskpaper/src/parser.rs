use std::sync::LazyLock;

use chrono::{Local, NaiveDateTime, TimeZone};
use regex::Regex;

use crate::{Document, Entry, Note, Section, Tag, Tags};

/// The default section name used when entries appear before any section header.
pub const DEFAULT_SECTION: &str = "Uncategorized";

static ENTRY_RX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^\t- (\d{4}-\d{2}-\d{2} \d{2}:\d{2}) \| (.*?)(?:\s+<([a-f0-9]{32})>)?\s*$").unwrap());
static SECTION_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\S[\S ]+):\s*$").unwrap());
static TAG_RX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^| )(@([^\s(]+)(?:\(([^)]+)\))?)").unwrap());

/// Flush the current entry (with accumulated note lines) into the current section.
fn flush_entry(current_section: &mut Option<Section>, current_entry: &mut Option<(Entry, Vec<String>)>) {
  if let Some((mut entry, note_lines)) = current_entry.take() {
    if !note_lines.is_empty() {
      *entry.note_mut() = Note::from_lines(note_lines);
    }
    if let Some(section) = current_section.as_mut() {
      section.add_entry(entry);
    }
  }
}

/// Flush the current section into the document.
fn flush_section(doc: &mut Document, current_section: &mut Option<Section>) {
  if let Some(section) = current_section.take() {
    doc.add_section(section);
  }
}

/// Parse a doing file string into a structured `Document`.
///
/// Recognizes section headers, entries with dates/tags/IDs, and notes.
/// Non-entry, non-section content is preserved as other content.
pub fn parse(content: &str) -> Document {
  let mut doc = Document::new();
  let mut current_section: Option<Section> = None;
  let mut current_entry: Option<(Entry, Vec<String>)> = None;
  let mut found_first_section = false;

  for line in content.lines() {
    if let Some(caps) = SECTION_RX.captures(line) {
      flush_entry(&mut current_section, &mut current_entry);
      flush_section(&mut doc, &mut current_section);
      found_first_section = true;
      current_section = Some(Section::new(&caps[1]));
      continue;
    }

    if let Some(caps) = ENTRY_RX.captures(line) {
      flush_entry(&mut current_section, &mut current_entry);

      if !found_first_section {
        found_first_section = true;
        current_section = Some(Section::new(DEFAULT_SECTION));
      }

      let date_str = &caps[1];
      let raw_title = caps[2].trim();
      let id = caps.get(3).map(|m| m.as_str());

      let section_name = current_section
        .as_ref()
        .map(|s| s.title().to_string())
        .unwrap_or_default();

      if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M")
        && let Some(date) = Local
          .from_local_datetime(&naive)
          .single()
          .or_else(|| Local.from_local_datetime(&naive).earliest())
          .or_else(|| Local.from_local_datetime(&naive).latest())
          .or_else(|| Some(naive.and_utc().with_timezone(&Local)))
      {
        let (title, tags) = parse_tags(raw_title);
        let entry = Entry::new(date, title, tags, Note::new(), &section_name, id);
        current_entry = Some((entry, Vec::new()));
      }
      continue;
    }

    if let Some(note_text) = line.strip_prefix("\t\t") {
      if let Some(ref mut entry) = current_entry {
        entry.1.push(note_text.to_string());
      }
      continue;
    }

    if !found_first_section {
      doc.other_content_top_mut().push(line.to_string());
    } else if let Some(section) = current_section.as_mut() {
      section.trailing_content_mut().push(line.to_string());
    } else {
      doc.other_content_bottom_mut().push(line.to_string());
    }
  }

  flush_entry(&mut current_section, &mut current_entry);
  flush_section(&mut doc, &mut current_section);

  doc
}

/// Extract tags from a title string, returning the tag-free title and a `Tags` collection.
fn parse_tags(title: &str) -> (String, Tags) {
  let mut tags = Vec::new();

  // Collect tag match byte ranges so we can build the cleaned title in one pass
  let mut tag_ranges: Vec<(usize, usize)> = Vec::new();
  for caps in TAG_RX.captures_iter(title) {
    let m = caps.get(1).unwrap();
    tag_ranges.push((m.start(), m.end()));
    let name = &caps[2];
    let value = caps.get(3).map(|m| m.as_str().to_string());
    tags.push(Tag::new(name, value));
  }

  // Build cleaned title by skipping tag ranges
  let mut clean = String::with_capacity(title.len());
  let mut pos = 0;
  for (start, end) in &tag_ranges {
    clean.push_str(&title[pos..*start]);
    pos = *end;
  }
  clean.push_str(&title[pos..]);

  let clean_title = clean.split_whitespace().collect::<Vec<_>>().join(" ");
  (clean_title, Tags::from_iter(tags))
}

#[cfg(test)]
mod test {
  use super::*;

  mod parse {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_handles_entries_without_section() {
      let content = "\t- 2024-03-17 14:30 | Orphan task";
      let doc = parse(content);

      assert!(doc.has_section(DEFAULT_SECTION));
      assert_eq!(doc.entries_in_section(DEFAULT_SECTION).count(), 1);
    }

    #[test]
    fn it_parses_empty_content() {
      let doc = parse("");

      assert!(doc.is_empty());
    }

    #[test]
    fn it_parses_empty_sections() {
      let content = "Currently:\nArchive:";
      let doc = parse(content);

      assert_eq!(doc.entries_in_section("Currently").count(), 0);
      assert_eq!(doc.entries_in_section("Archive").count(), 0);
    }

    #[test]
    fn it_parses_entries_with_dates_and_titles() {
      let content = "Currently:\n\t- 2024-03-17 14:30 | Working on feature";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Working on feature");
      assert_eq!(
        entries[0].date(),
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap()
      );
    }

    #[test]
    fn it_parses_entries_with_ids() {
      let content = "Currently:\n\t- 2024-03-17 14:30 | Working on feature <aaaabbbbccccddddeeeeffffaaaabbbb>";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries[0].id(), "aaaabbbbccccddddeeeeffffaaaabbbb");
    }

    #[test]
    fn it_parses_entries_with_tags() {
      let content = "Currently:\n\t- 2024-03-17 14:30 | Working on feature @coding @done(2024-03-17 15:00)";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries[0].title(), "Working on feature");
      assert!(entries[0].tags().has("coding"));
      assert!(entries[0].tags().has("done"));
      assert_eq!(
        entries[0].tags().iter().find(|t| t.name() == "done").unwrap().value(),
        Some("2024-03-17 15:00")
      );
    }

    #[test]
    fn it_parses_entry_with_tags_and_id() {
      let content =
        "Currently:\n\t- 2024-03-17 14:30 | My task @flag @done(2024-03-17 15:00) <aaaabbbbccccddddeeeeffffaaaabbbb>";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries[0].title(), "My task");
      assert!(entries[0].tags().has("flag"));
      assert!(entries[0].tags().has("done"));
      assert_eq!(entries[0].id(), "aaaabbbbccccddddeeeeffffaaaabbbb");
    }

    #[test]
    fn it_parses_multiple_sections_with_entries() {
      let content = "\
Currently:
\t- 2024-03-17 14:30 | Task A @coding
\t- 2024-03-17 15:00 | Task B
Archive:
\t- 2024-03-16 10:00 | Old task @done(2024-03-16 11:00)";
      let doc = parse(content);

      assert_eq!(doc.len(), 2);
      assert_eq!(doc.entries_in_section("Currently").count(), 2);
      assert_eq!(doc.entries_in_section("Archive").count(), 1);
    }

    #[test]
    fn it_parses_notes() {
      let content = "Currently:\n\t- 2024-03-17 14:30 | Working on feature\n\t\tA note line\n\t\tAnother note";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries[0].note().len(), 2);
      assert_eq!(entries[0].note().lines(), &["A note line", "Another note"]);
    }

    #[test]
    fn it_parses_section_headers() {
      let content = "Currently:\nArchive:";
      let doc = parse(content);

      assert_eq!(doc.len(), 2);
      let names: Vec<&str> = doc.sections().iter().map(|s| s.title()).collect();
      assert_eq!(names, vec!["Currently", "Archive"]);
    }

    #[test]
    fn it_preserves_other_content_top() {
      let content = "# My Doing File\n\nCurrently:";
      let doc = parse(content);

      assert_eq!(doc.other_content_top(), &["# My Doing File", ""]);
      assert!(doc.has_section("Currently"));
    }

    #[test]
    fn it_generates_id_when_none_present() {
      let content = "Currently:\n\t- 2024-03-17 14:30 | Working on feature";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries[0].id().len(), 32);
      assert!(entries[0].id().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn it_round_trips_a_document() {
      let content = "\
Currently:
\t- 2024-03-17 14:30 | Working on feature @coding <aaaabbbbccccddddeeeeffffaaaabbbb>
\t\tA note about the work
Archive:
\t- 2024-03-16 10:00 | Old task @done(2024-03-16 11:00) <bbbbccccddddeeeeffffaaaabbbbcccc>";
      let doc = parse(content);
      let output = format!("{doc}");

      assert_eq!(output, content);
    }

    #[test]
    fn it_round_trips_with_other_content() {
      let content = "\
# My Doing File
Currently:
\t- 2024-03-17 14:30 | Task A <aaaabbbbccccddddeeeeffffaaaabbbb>";
      let doc = parse(content);

      assert_eq!(doc.other_content_top(), &["# My Doing File"]);

      let output = format!("{doc}");

      assert_eq!(
        output,
        "# My Doing File\n\nCurrently:\n\t- 2024-03-17 14:30 | Task A <aaaabbbbccccddddeeeeffffaaaabbbb>"
      );
    }

    #[test]
    fn it_merges_duplicate_section_headers() {
      let content = "\
Archive:
\t- 2024-03-16 10:00 | Old task @done(2024-03-16 11:00)
Archive:
\t- 2024-03-17 09:00 | Another old task @done(2024-03-17 10:00)";
      let doc = parse(content);

      assert_eq!(doc.len(), 1);
      assert_eq!(doc.entries_in_section("Archive").count(), 2);
    }

    #[test]
    fn it_preserves_entries_with_dst_ambiguous_timestamps() {
      // 2024-03-10 02:30 falls in the US spring-forward DST gap (2:00 AM → 3:00 AM).
      // 2024-11-03 01:30 falls in the US fall-back DST fold (1:00 AM occurs twice).
      // Regardless of the test machine's timezone, these entries must never be dropped.
      let content = "\
Currently:
\t- 2024-03-10 02:30 | Spring forward task
\t- 2024-11-03 01:30 | Fall back task
\t- 2024-06-15 14:00 | Normal task";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries.len(), 3);
      assert_eq!(entries[0].title(), "Spring forward task");
      assert_eq!(entries[1].title(), "Fall back task");
      assert_eq!(entries[2].title(), "Normal task");
    }

    #[test]
    fn it_preserves_inter_section_content_position() {
      let content = "\
Currently:
\t- 2024-03-17 14:30 | Task A <aaaabbbbccccddddeeeeffffaaaabbbb>
# A comment between sections
Archive:
\t- 2024-03-16 10:00 | Task B <bbbbccccddddeeeeffffaaaabbbbcccc>";
      let doc = parse(content);

      let currently = &doc.sections()[0];
      assert_eq!(currently.trailing_content(), &["# A comment between sections"]);
      assert!(doc.other_content_bottom().is_empty());
    }

    #[test]
    fn it_round_trips_document_with_comments_between_sections() {
      let content = "\
Currently:
\t- 2024-03-17 14:30 | Task A <aaaabbbbccccddddeeeeffffaaaabbbb>
# A comment between sections
Archive:
\t- 2024-03-16 10:00 | Task B <bbbbccccddddeeeeffffaaaabbbbcccc>";
      let doc = parse(content);
      let output = format!("{doc}");

      assert_eq!(output, content);
    }

    #[test]
    fn it_only_puts_actual_bottom_content_in_other_content_bottom() {
      let content = "\
Currently:
\t- 2024-03-17 14:30 | Task A <aaaabbbbccccddddeeeeffffaaaabbbb>
Archive:
\t- 2024-03-16 10:00 | Task B <bbbbccccddddeeeeffffaaaabbbbcccc>";
      let doc = parse(content);

      assert!(doc.other_content_bottom().is_empty());
    }

    #[test]
    fn it_skips_malformed_lines_gracefully() {
      let content = "Currently:\n\t- not a valid entry\n\t- 2024-03-17 14:30 | Valid task";
      let doc = parse(content);

      let entries: Vec<_> = doc.entries_in_section("Currently").collect();
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Valid task");
    }
  }

  mod parse_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_extracts_multiple_tags() {
      let (title, tags) = parse_tags("My task @coding @important @done(2024-03-17 15:00)");

      assert_eq!(title, "My task");
      assert_eq!(tags.len(), 3);
      assert!(tags.has("coding"));
      assert!(tags.has("important"));
      assert!(tags.has("done"));
    }

    #[test]
    fn it_extracts_simple_tags() {
      let (title, tags) = parse_tags("Working on feature @coding");

      assert_eq!(title, "Working on feature");
      assert_eq!(tags.len(), 1);
      assert!(tags.has("coding"));
    }

    #[test]
    fn it_extracts_tags_with_values() {
      let (title, tags) = parse_tags("Task @done(2024-03-17 15:00)");

      assert_eq!(title, "Task");
      assert_eq!(tags.len(), 1);
      assert_eq!(tags.iter().next().unwrap().value(), Some("2024-03-17 15:00"));
    }

    #[test]
    fn it_handles_tags_in_middle_of_title() {
      let (title, tags) = parse_tags("Start @flag end");

      assert_eq!(title, "Start end");
      assert!(tags.has("flag"));
    }

    #[test]
    fn it_returns_empty_tags_for_no_tags() {
      let (title, tags) = parse_tags("Just a plain title");

      assert_eq!(title, "Just a plain title");
      assert!(tags.is_empty());
    }
  }
}
