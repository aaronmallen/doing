use std::path::{Path, PathBuf};

use chrono::Local;
use clap::Args;
use doing_taskpaper::{Document, Entry, Section, io as taskpaper_io};

use crate::{
  Result,
  cli::{AppContext, args::FilterArgs},
  ops::backup::write_with_backup,
};

/// Move entries to a dated archive file.
///
/// Moves entries from the specified section to a dated archive file stored
/// alongside the main doing file. The archive file is named using the pattern
/// `{doing_file_stem}_{YYYY-MM-DD}.md`.
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(flatten)]
  filter: FilterArgs,

  /// Number of entries to keep in the source section
  #[arg(short, long)]
  keep: Option<usize>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let archive_path = self.archive_path(&ctx.doing_file);

    let section_names: Vec<String> = if self.filter.section.is_some() {
      vec![
        self
          .filter
          .section
          .clone()
          .unwrap_or_else(|| ctx.config.current_section.clone()),
      ]
    } else {
      ctx.document.sections().iter().map(|s| s.title().to_string()).collect()
    };

    let mut all_entries_to_rotate = Vec::new();
    for section_name in &section_names {
      let entries = self.find_entries(ctx, section_name)?;
      if !entries.is_empty() {
        self.write_to_archive(&archive_path, &entries, section_name, ctx)?;
        all_entries_to_rotate.extend(entries);
      }
    }

    if all_entries_to_rotate.is_empty() {
      ctx.status("No entries to rotate");
      return Ok(());
    }

    let rotated_count = all_entries_to_rotate.len();
    self.remove_from_source(ctx, &all_entries_to_rotate)?;

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    if rotated_count == 1 {
      ctx.status(format!("Rotated 1 entry to {}", archive_path.display()));
    } else {
      ctx.status(format!("Rotated {rotated_count} entries to {}", archive_path.display()));
    }

    Ok(())
  }

  fn archive_path(&self, doing_file: &Path) -> PathBuf {
    let stem = doing_file.file_stem().and_then(|s| s.to_str()).unwrap_or("doing");
    let date_suffix = Local::now().format("%Y-%m-%d");
    let archive_name = format!("{stem}_{date_suffix}.md");

    doing_file
      .parent()
      .map(|p| p.join(&archive_name))
      .unwrap_or_else(|| PathBuf::from(&archive_name))
  }

  fn find_entries(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<Entry>> {
    let all_entries: Vec<Entry> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    if all_entries.is_empty() {
      return Ok(Vec::new());
    }

    // Apply full filter pipeline
    let has_filters = self.filter.before.is_some()
      || self.filter.after.is_some()
      || self.filter.from.is_some()
      || self.filter.search.is_some()
      || !self.filter.tag.is_empty()
      || !self.filter.val.is_empty()
      || self.filter.case.is_some()
      || self.filter.exact
      || self.filter.not;

    let mut candidates = if has_filters {
      let filter_args = FilterArgs {
        section: Some(section_name.to_string()),
        ..self.filter.clone()
      };
      let options = filter_args.into_filter_options(&ctx.config, ctx.include_notes)?;
      crate::ops::filter::filter_entries(all_entries, &options)
    } else {
      all_entries
    };

    // Sort oldest-first for keep logic
    candidates.sort_by_key(|e| e.date());

    // Apply --keep: skip the N newest entries
    if let Some(keep) = self.keep {
      if keep >= candidates.len() {
        return Ok(Vec::new());
      }
      candidates.truncate(candidates.len() - keep);
    }

    Ok(candidates)
  }

  fn remove_from_source(&self, ctx: &mut AppContext, entries: &[Entry]) -> Result<()> {
    let ids: Vec<&str> = entries.iter().map(|e| e.id()).collect();

    for section in ctx.document.sections_mut() {
      section.entries_mut().retain(|e| !ids.contains(&e.id()));
    }

    Ok(())
  }

  fn write_to_archive(
    &self,
    archive_path: &Path,
    entries: &[Entry],
    section_name: &str,
    ctx: &AppContext,
  ) -> Result<()> {
    let mut archive_doc = if archive_path.exists() {
      taskpaper_io::read_file(archive_path)?
    } else {
      Document::new()
    };

    if !archive_doc.has_section(section_name) {
      archive_doc.add_section(Section::new(section_name));
    }

    let archive_section = archive_doc.section_by_name_mut(section_name).unwrap();
    for entry in entries {
      archive_section.add_entry(entry.clone());
    }

    archive_doc.sort_entries(ctx.config.doing_file_sort == doing_config::SortOrder::Desc);
    taskpaper_io::write_file(&archive_doc, archive_path)?;

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};
  use doing_config::Config;
  use doing_taskpaper::{Note, Tag, Tags};

  use super::*;
  use crate::cli::args::FilterArgs;

  fn default_cmd() -> Command {
    Command {
      filter: FilterArgs::default(),
      keep: None,
    }
  }

  fn sample_ctx_with_done(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "Done task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 14:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Active task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: doc,
      doing_file: path,
      include_notes: true,
      no: false,
      noauto: false,
      quiet: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  fn sample_ctx_with_multiple_done(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 10, 0, 0).unwrap(),
      "First done",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 11:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 12, 0, 0).unwrap(),
      "Second done",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 13:00"))]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Active task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: Config::default(),
      default_answer: false,
      document: doc,
      doing_file: path,
      include_notes: true,
      no: false,
      noauto: false,
      quiet: false,
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod archive_path {
    use super::*;

    #[test]
    fn it_generates_dated_archive_filename() {
      let cmd = default_cmd();
      let doing_file = PathBuf::from("/tmp/doing.md");

      let path = cmd.archive_path(&doing_file);

      let expected_suffix = Local::now().format("%Y-%m-%d").to_string();
      let name = path.file_name().unwrap().to_str().unwrap();
      assert!(name.starts_with("doing_"));
      assert!(name.contains(&expected_suffix));
      assert!(name.ends_with(".md"));
      assert_eq!(path.parent().unwrap().to_str().unwrap(), "/tmp");
    }
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_creates_archive_file() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let archive_path = cmd.archive_path(&ctx.doing_file);
      assert!(archive_path.exists());
      let archive_doc = taskpaper_io::read_file(&archive_path).unwrap();
      let entries = archive_doc.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
    }

    #[test]
    fn it_does_nothing_when_section_is_empty() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut doc = Document::new();
      doc.add_section(Section::new("Currently"));
      let mut ctx = AppContext {
        config: Config::default(),
        default_answer: false,
        document: doc,
        doing_file: path,
        include_notes: true,
        no: false,
        noauto: false,
        quiet: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.entries_in_section("Currently").is_empty());
    }

    #[test]
    fn it_keeps_entries_in_source_section() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple_done(dir.path());
      let cmd = Command {
        keep: Some(1),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let currently = ctx.document.entries_in_section("Currently");
      assert_eq!(currently.len(), 1);
      assert_eq!(currently[0].title(), "Active task");

      let archive_path = cmd.archive_path(&ctx.doing_file);
      let archive_doc = taskpaper_io::read_file(&archive_path).unwrap();
      let archived = archive_doc.entries_in_section("Currently");
      assert_eq!(archived.len(), 2);
    }

    #[test]
    fn it_moves_entries_before_date() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple_done(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          before: Some("2024-03-17 11:00".into()),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      assert_eq!(ctx.document.entries_in_section("Currently").len(), 2);

      let archive_path = cmd.archive_path(&ctx.doing_file);
      let archive_doc = taskpaper_io::read_file(&archive_path).unwrap();
      let archived = archive_doc.entries_in_section("Currently");
      assert_eq!(archived.len(), 1);
      assert_eq!(archived[0].title(), "First done");
    }

    #[test]
    fn it_removes_rotated_entries_from_source() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      assert!(ctx.document.entries_in_section("Currently").is_empty());
    }
  }
}
