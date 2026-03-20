use chrono::Local;
use clap::{ArgAction, Args};
use log::info;

use crate::{
  cli::{AppContext, args::FilterArgs},
  errors::Result,
  ops::{
    backup::write_with_backup,
    filter::{Age, filter_entries},
  },
  taskpaper::Entry,
  time::chronify,
};

/// Reset the start date of the last entry to now.
///
/// By default, sets the start date of the last entry to the current time and
/// removes the @done tag, effectively resuming the task. Use --no-resume to
/// keep the @done tag. Use --back to set a specific start date.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Set a specific start date (natural language)
  #[arg(short, long)]
  back: Option<String>,

  #[command(flatten)]
  filter: FilterArgs,

  #[arg(long = "no-resume", action = ArgAction::SetTrue, hide = true, overrides_with = "resume")]
  no_resume: bool,

  /// Remove @done tag to re-open the entry
  #[arg(short, long, action = ArgAction::SetTrue, overrides_with = "no_resume", default_value_t = true)]
  resume: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let entries = self.find_entries(ctx)?;

    if entries.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    let new_date = self.resolve_date()?;

    for loc in &entries {
      self.reset_entry(ctx, loc, new_date)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let count = entries.len();
    if count == 1 {
      info!("Reset 1 entry");
    } else {
      info!("Reset {count} entries");
    }

    Ok(())
  }

  fn find_entries(&self, ctx: &AppContext) -> Result<Vec<EntryLocation>> {
    let section = self
      .filter
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let has_filters = !self.filter.tag.is_empty() || self.filter.search.is_some();

    if has_filters {
      let all_entries: Vec<Entry> = ctx.document.all_entries().into_iter().cloned().collect();

      let mut options = self
        .filter
        .clone()
        .into_filter_options(&ctx.config, ctx.include_notes)?;
      options.age = options.age.or(Some(Age::Newest));

      let results = filter_entries(all_entries, &options);
      return Ok(
        results
          .iter()
          .map(|e| EntryLocation {
            id: e.id().to_string(),
            section: e.section().to_string(),
          })
          .collect(),
      );
    }

    let count = self.filter.count.unwrap_or(1);
    let entries = ctx.document.entries_in_section(&section);
    let mut locs: Vec<EntryLocation> = entries
      .iter()
      .rev()
      .take(count)
      .map(|e| EntryLocation {
        id: e.id().to_string(),
        section: e.section().to_string(),
      })
      .collect();
    locs.reverse();

    Ok(locs)
  }

  fn find_entry_mut<'a>(&self, ctx: &'a mut AppContext, loc: &EntryLocation) -> Result<&'a mut Entry> {
    let section = ctx
      .document
      .section_by_name_mut(&loc.section)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{}\" not found", loc.section)))?;

    section
      .entries_mut()
      .iter_mut()
      .find(|e| e.id() == loc.id)
      .ok_or_else(|| crate::errors::Error::Config("entry not found".into()))
  }

  fn reset_entry(&self, ctx: &mut AppContext, loc: &EntryLocation, new_date: chrono::DateTime<Local>) -> Result<()> {
    let entry = self.find_entry_mut(ctx, loc)?;
    entry.set_date(new_date);

    if self.resume && !self.no_resume {
      entry.tags_mut().remove("done");
    }

    Ok(())
  }

  fn resolve_date(&self) -> Result<chrono::DateTime<Local>> {
    match &self.back {
      Some(expr) => chronify(expr),
      None => Ok(Local::now()),
    }
  }
}

/// Tracks an entry's ID and section for locating it in the document.
#[derive(Clone, Debug)]
struct EntryLocation {
  id: String,
  section: String,
}

#[cfg(test)]
mod test {
  use std::fs;

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    cli::args::FilterArgs,
    config::Config,
    taskpaper::{Document, Note, Section, Tag, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      back: None,
      filter: FilterArgs::default(),
      no_resume: false,
      resume: true,
    }
  }

  fn sample_ctx(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
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
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  fn sample_ctx_with_done(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Done task",
      Tags::from_iter(vec![Tag::new("done", Some("2024-03-17 15:00"))]),
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
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  fn sample_ctx_with_multiple(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "First task",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Second task",
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
      stdout: false,
      use_color: false,
      use_pager: false,
      yes: false,
    }
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_errors_on_empty_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut ctx = AppContext {
        config: Config::default(),
        default_answer: false,
        document: Document::parse("Currently:\n"),
        doing_file: path,
        include_notes: true,
        no: false,
        noauto: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_keeps_done_tag_with_no_resume() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = Command {
        no_resume: true,
        resume: false,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].finished());
    }

    #[test]
    fn it_removes_done_tag_by_default() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_done(dir.path());
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(!entries[0].finished());
    }

    #[test]
    fn it_resets_multiple_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      let cmd = Command {
        filter: FilterArgs {
          count: Some(2),
          ..Default::default()
        },
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      for entry in entries {
        let elapsed = Local::now().signed_duration_since(entry.date());
        assert!(elapsed.num_seconds() < 5);
      }
    }

    #[test]
    fn it_resets_start_date_to_now() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let original_date = ctx.document.entries_in_section("Currently")[0].date();
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_ne!(entries[0].date(), original_date);
      // New date should be very recent (within last few seconds)
      let elapsed = Local::now().signed_duration_since(entries[0].date());
      assert!(elapsed.num_seconds() < 5);
    }

    #[test]
    fn it_resets_with_back_date() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      let cmd = Command {
        back: Some("2024-06-15 10:00".into()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      let expected = Local.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
      assert_eq!(entries[0].date(), expected);
    }
  }

  mod set_date {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_updates_entry_date() {
      let mut entry = Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "test",
        Tags::new(),
        Note::new(),
        "Currently",
        None::<String>,
      );
      let new_date = Local.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();

      entry.set_date(new_date);

      assert_eq!(entry.date(), new_date);
    }
  }
}
