use std::path::PathBuf;

use clap::Args;
use log::info;

use crate::{
  cli::AppContext,
  errors::Result,
  ops::{autotag::autotag, backup::write_with_backup},
  plugins::import,
  taskpaper::{Entry, Section, Tag},
};

/// Import entries from other doing files or Timing.app JSON exports.
///
/// Reads entries from a source file, optionally filters them, applies tags
/// or prefixes, and merges them into the current doing file.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Apply autotagging rules to imported entries
  #[arg(long)]
  autotag: bool,

  /// Date range filter for imports (e.g. "last week", "2024-01-01 to 2024-03-01")
  #[arg(long)]
  from: Option<String>,

  /// Import format type (doing, timing)
  #[arg(short = 't', long = "type")]
  import_type: Option<String>,

  /// Skip entries whose time range overlaps with existing entries
  #[arg(long)]
  no_overlap: bool,

  /// Path to the file to import
  path: PathBuf,

  /// Prepend text to all imported entry titles
  #[arg(long)]
  prefix: Option<String>,

  /// Filter which entries to import by search query
  #[arg(long)]
  search: Option<String>,

  /// Target section for imported entries (default: current section)
  #[arg(short, long)]
  section: Option<String>,

  /// Apply additional tags to all imported entries
  #[arg(short, long, value_delimiter = ',')]
  tag: Vec<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let registry = import::default_registry();
    let format = self.resolve_format()?;
    let plugin = registry.resolve(&format).ok_or_else(|| {
      let available = registry.available_formats().join(", ");
      crate::errors::Error::Plugin(format!("unknown import format \"{format}\". Available: {available}"))
    })?;

    let mut entries = plugin.import(&self.path)?;

    if entries.is_empty() {
      info!("No entries found in {}", self.path.display());
      return Ok(());
    }

    self.apply_search_filter(&mut entries)?;
    self.apply_date_filter(&mut entries)?;

    let section_name = self.section.as_deref().unwrap_or(&ctx.config.current_section);

    let mut imported = 0;
    for mut entry in entries {
      if self.no_overlap && has_overlap(&entry, ctx) {
        continue;
      }

      self.apply_prefix(&mut entry);
      self.apply_tags(&mut entry);

      if self.autotag {
        autotag(&mut entry, &ctx.config.autotag, &ctx.config.default_tags);
      }

      if !ctx.document.has_section(section_name) {
        ctx.document.add_section(Section::new(section_name));
      }
      ctx.document.section_by_name_mut(section_name).unwrap().add_entry(entry);
      imported += 1;
    }

    ctx.document.dedup();
    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    info!("Imported {imported} entries into {section_name}");
    Ok(())
  }

  fn apply_date_filter(&self, entries: &mut Vec<Entry>) -> Result<()> {
    if let Some(ref from) = self.from {
      let (after, before) = crate::time::parse_range(from)?;
      entries.retain(|e| {
        let date = e.date();
        date >= after && date <= before
      });
    }
    Ok(())
  }

  fn apply_prefix(&self, entry: &mut Entry) {
    if let Some(ref prefix) = self.prefix {
      let new_title = format!("{prefix} {}", entry.title());
      *entry = Entry::new(
        entry.date(),
        &new_title,
        entry.tags().clone(),
        entry.note().clone(),
        entry.section(),
        Some(entry.id()),
      );
    }
  }

  fn apply_search_filter(&self, entries: &mut Vec<Entry>) -> Result<()> {
    if let Some(ref query) = self.search {
      let (mode, case) = crate::ops::search::parse_query(query, &Default::default())
        .ok_or_else(|| crate::errors::Error::Parse(format!("invalid search query: {query}")))?;
      entries.retain(|e| crate::ops::search::matches_entry(e, &mode, case, true));
    }
    Ok(())
  }

  fn apply_tags(&self, entry: &mut Entry) {
    for tag_name in &self.tag {
      let name = tag_name.strip_prefix('@').unwrap_or(tag_name);
      if !entry.tags().has(name) {
        entry.tags_mut().add(Tag::new(name, None::<String>));
      }
    }
  }

  /// Determine the import format from `--type` flag or file extension.
  fn resolve_format(&self) -> Result<String> {
    if let Some(ref t) = self.import_type {
      return Ok(t.clone());
    }

    let ext = self.path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
      "json" => Ok("timing".into()),
      _ => Ok("doing".into()),
    }
  }
}

/// Check whether an entry's time range overlaps with any existing entry.
fn has_overlap(entry: &Entry, ctx: &AppContext) -> bool {
  ctx
    .document
    .all_entries()
    .iter()
    .any(|existing| entry.overlapping_time(existing))
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    config::Config,
    taskpaper::{Document, Entry, Note, Section, Tag, Tags},
  };

  fn sample_ctx(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    AppContext {
      config: Config::default(),
      document: Document::parse("Currently:\n"),
      doing_file: path,
    }
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_imports_from_doing_file() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let source = dir.path().join("source.md");
      fs::write(
        &source,
        "Currently:\n\t- 2024-03-17 14:30 | Imported task <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
      )
      .unwrap();
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("doing".into()),
        no_overlap: false,
        path: source,
        prefix: None,
        search: None,
        section: None,
        tag: vec![],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Imported task");
    }

    #[test]
    fn it_imports_into_custom_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let source = dir.path().join("source.md");
      fs::write(
        &source,
        "Currently:\n\t- 2024-03-17 14:30 | Task <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
      )
      .unwrap();
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("doing".into()),
        no_overlap: false,
        path: source,
        prefix: None,
        search: None,
        section: Some("Archive".into()),
        tag: vec![],
      };

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.has_section("Archive"));
      let entries = ctx.document.entries_in_section("Archive");
      assert_eq!(entries.len(), 1);
    }

    #[test]
    fn it_applies_prefix_to_imported_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let source = dir.path().join("source.md");
      fs::write(
        &source,
        "Currently:\n\t- 2024-03-17 14:30 | Task <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
      )
      .unwrap();
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("doing".into()),
        no_overlap: false,
        path: source,
        prefix: Some("[imported]".into()),
        search: None,
        section: None,
        tag: vec![],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries[0].title(), "[imported] Task");
    }

    #[test]
    fn it_applies_tags_to_imported_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let source = dir.path().join("source.md");
      fs::write(
        &source,
        "Currently:\n\t- 2024-03-17 14:30 | Task <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
      )
      .unwrap();
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("doing".into()),
        no_overlap: false,
        path: source,
        prefix: None,
        search: None,
        section: None,
        tag: vec!["imported".into(), "@work".into()],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("imported"));
      assert!(entries[0].tags().has("work"));
    }

    #[test]
    fn it_skips_overlapping_entries_with_no_overlap() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Existing task",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);
      let mut ctx = AppContext {
        config: Config::default(),
        document: doc,
        doing_file: path,
      };

      let source = dir.path().join("source.md");
      fs::write(
        &source,
        "Currently:\n\t- 2024-03-17 14:30 | Overlapping task @done(2024-03-17 15:30) <aaaabbbbccccddddeeeeffffaaaabbbb>\n",
      )
      .unwrap();
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("doing".into()),
        no_overlap: true,
        path: source,
        prefix: None,
        search: None,
        section: None,
        tag: vec![],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "Existing task");
    }

    #[test]
    fn it_returns_ok_for_empty_source() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let source = dir.path().join("empty.md");
      fs::write(&source, "").unwrap();
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("doing".into()),
        no_overlap: false,
        path: source,
        prefix: None,
        search: None,
        section: None,
        tag: vec![],
      };

      assert!(cmd.call(&mut ctx).is_ok());
    }

    #[test]
    fn it_imports_from_timing_json() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let source = dir.path().join("timing.json");
      fs::write(
        &source,
        r#"[
          {
            "activityTitle": "Writing code",
            "activityType": "Task",
            "startDate": "2024-03-17 14:00",
            "endDate": "2024-03-17 15:00",
            "project": "Work",
            "notes": null
          }
        ]"#,
      )
      .unwrap();
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("timing".into()),
        no_overlap: false,
        path: source,
        prefix: None,
        search: None,
        section: None,
        tag: vec![],
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 1);
      assert_eq!(entries[0].title(), "[Timing.app] Writing code");
    }
  }

  mod has_overlap {
    use super::*;

    #[test]
    fn it_detects_overlapping_entries() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Existing",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);
      let ctx = AppContext {
        config: Config::default(),
        document: doc,
        doing_file: path,
      };

      let entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 30, 0).unwrap(),
        "New",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:30"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(super::has_overlap(&entry, &ctx));
    }

    #[test]
    fn it_returns_false_for_non_overlapping() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut doc = Document::new();
      let mut section = Section::new("Currently");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Existing",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      ));
      doc.add_section(section);
      let ctx = AppContext {
        config: Config::default(),
        document: doc,
        doing_file: path,
      };

      let entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 16, 0, 0).unwrap(),
        "New",
        Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 17:00"))]),
        Note::new(),
        "Currently",
        None::<String>,
      );

      assert!(!super::has_overlap(&entry, &ctx));
    }
  }

  mod resolve_format {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_uses_explicit_type() {
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: Some("timing".into()),
        no_overlap: false,
        path: PathBuf::from("file.md"),
        prefix: None,
        search: None,
        section: None,
        tag: vec![],
      };

      assert_eq!(cmd.resolve_format().unwrap(), "timing");
    }

    #[test]
    fn it_infers_timing_from_json_extension() {
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: None,
        no_overlap: false,
        path: PathBuf::from("export.json"),
        prefix: None,
        search: None,
        section: None,
        tag: vec![],
      };

      assert_eq!(cmd.resolve_format().unwrap(), "timing");
    }

    #[test]
    fn it_defaults_to_doing_format() {
      let cmd = Command {
        autotag: false,
        from: None,
        import_type: None,
        no_overlap: false,
        path: PathBuf::from("other.md"),
        prefix: None,
        search: None,
        section: None,
        tag: vec![],
      };

      assert_eq!(cmd.resolve_format().unwrap(), "doing");
    }
  }
}
