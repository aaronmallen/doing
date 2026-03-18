use clap::Args;
use log::info;

use crate::{
  cli::AppContext,
  errors::Result,
  ops::{autotag, backup::write_with_backup},
};

/// Apply autotagging rules to existing entries.
///
/// Retroactively applies the configured autotag rules (whitelist, synonyms,
/// and transforms) plus default tags to entries that were created before
/// those rules existed or with autotagging disabled.
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Apply to the last N entries
  #[arg(short, long, default_value_t = 1)]
  count: usize,

  /// Section to target
  #[arg(short = 'S', long)]
  section: Option<String>,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self
      .section
      .clone()
      .unwrap_or_else(|| ctx.config.current_section.clone());

    let entry_ids = self.find_entries(ctx, &section_name)?;

    if entry_ids.is_empty() {
      return Err(crate::errors::Error::Config("no matching entries found".into()));
    }

    for (id, section) in &entry_ids {
      self.autotag_entry(ctx, section, id)?;
    }

    write_with_backup(&ctx.document, &ctx.doing_file, &ctx.config)?;

    let count = entry_ids.len();
    if count == 1 {
      info!("Autotagged 1 entry");
    } else {
      info!("Autotagged {count} entries");
    }

    Ok(())
  }

  fn autotag_entry(&self, ctx: &mut AppContext, section_name: &str, entry_id: &str) -> Result<()> {
    let autotag_config = ctx.config.autotag.clone();
    let default_tags = ctx.config.default_tags.clone();

    let section = ctx
      .document
      .section_by_name_mut(section_name)
      .ok_or_else(|| crate::errors::Error::Config(format!("section \"{section_name}\" not found")))?;

    let entry = section
      .entries_mut()
      .iter_mut()
      .find(|e| e.id() == entry_id)
      .ok_or_else(|| crate::errors::Error::Config("entry not found".into()))?;

    autotag::autotag(entry, &autotag_config, &default_tags);

    Ok(())
  }

  fn find_entries(&self, ctx: &AppContext, section_name: &str) -> Result<Vec<(String, String)>> {
    let entries = ctx.document.entries_in_section(section_name);
    let mut ids: Vec<(String, String)> = entries
      .iter()
      .rev()
      .take(self.count)
      .map(|e| (e.id().to_string(), e.section().to_string()))
      .collect();
    ids.reverse();

    Ok(ids)
  }
}

#[cfg(test)]
mod test {
  use std::{collections::HashMap, fs};

  use chrono::{Local, TimeZone};

  use super::*;
  use crate::{
    config::Config,
    taskpaper::{Document, Entry, Note, Section, Tags},
  };

  fn default_cmd() -> Command {
    Command {
      count: 1,
      section: None,
    }
  }

  fn sample_ctx(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Working on design",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: Config::default(),
      document: doc,
      doing_file: path,
    }
  }

  fn sample_ctx_with_multiple(dir: &std::path::Path) -> AppContext {
    let path = dir.join("doing.md");
    fs::write(&path, "Currently:\n").unwrap();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 13, 0, 0).unwrap(),
      "Working on design",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Reviewing code",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);
    AppContext {
      config: Config::default(),
      document: doc,
      doing_file: path,
    }
  }

  mod call {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_applies_default_tags() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.default_tags = vec!["tracked".to_string()];
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("tracked"));
    }

    #[test]
    fn it_applies_whitelist_rules() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.autotag.whitelist = vec!["design".to_string()];
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("design"));
    }

    #[test]
    fn it_applies_synonym_rules() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.autotag.synonyms = {
        let mut m = HashMap::new();
        m.insert("creative".to_string(), vec!["design".to_string()]);
        m
      };
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert!(entries[0].tags().has("creative"));
    }

    #[test]
    fn it_applies_to_last_n_entries() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx_with_multiple(dir.path());
      ctx.config.default_tags = vec!["tracked".to_string()];
      let cmd = Command {
        count: 2,
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Currently");
      assert_eq!(entries.len(), 2);
      assert!(entries[0].tags().has("tracked"));
      assert!(entries[1].tags().has("tracked"));
    }

    #[test]
    fn it_errors_on_empty_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut ctx = AppContext {
        config: Config::default(),
        document: Document::parse("Currently:\n"),
        doing_file: path,
      };
      let cmd = default_cmd();

      assert!(cmd.call(&mut ctx).is_err());
    }

    #[test]
    fn it_is_idempotent() {
      let dir = tempfile::tempdir().unwrap();
      let mut ctx = sample_ctx(dir.path());
      ctx.config.autotag.whitelist = vec!["design".to_string()];
      ctx.config.default_tags = vec!["tracked".to_string()];
      let cmd = default_cmd();

      cmd.call(&mut ctx).unwrap();
      let tags_after_first = ctx.document.entries_in_section("Currently")[0].tags().len();

      cmd.call(&mut ctx).unwrap();
      let tags_after_second = ctx.document.entries_in_section("Currently")[0].tags().len();

      assert_eq!(tags_after_first, tags_after_second);
    }

    #[test]
    fn it_targets_specified_section() {
      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("doing.md");
      fs::write(&path, "Currently:\n").unwrap();
      let mut doc = Document::new();
      let mut section = Section::new("Work");
      section.add_entry(Entry::new(
        Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
        "Working on design",
        Tags::new(),
        Note::new(),
        "Work",
        None::<String>,
      ));
      doc.add_section(section);
      doc.add_section(Section::new("Currently"));
      let mut ctx = AppContext {
        config: Config::default(),
        document: doc,
        doing_file: path,
      };
      ctx.config.autotag.whitelist = vec!["design".to_string()];
      let cmd = Command {
        section: Some("Work".to_string()),
        ..default_cmd()
      };

      cmd.call(&mut ctx).unwrap();

      let entries = ctx.document.entries_in_section("Work");
      assert!(entries[0].tags().has("design"));
    }
  }
}
