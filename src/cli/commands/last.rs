use clap::Args;

use crate::{
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    pager,
  },
  config::SortOrder,
  errors::Result,
  ops::filter::{Age, filter_entries},
  template::renderer::{RenderOptions, format_items},
};

/// Show the single most recent entry.
///
/// Displays the last entry added to the doing file. Use `--section` or
/// `--tag` to pick which "last" entry to show. Defaults to the `last`
/// template from config.
///
/// # Examples
///
/// ```text
/// doing last                    # most recent entry
/// doing last -S Later           # most recent in "Later"
/// doing last --tag meeting      # most recent tagged @meeting
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  #[command(flatten)]
  display: DisplayArgs,

  #[command(flatten)]
  filter: FilterArgs,

  /// Use a pager for output
  #[arg(short, long)]
  pager: bool,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let section_name = self.filter.section.as_deref().unwrap_or("all");

    let all_entries: Vec<_> = ctx
      .document
      .entries_in_section(section_name)
      .into_iter()
      .cloned()
      .collect();

    let mut options = self.filter.clone().into_filter_options(&ctx.config)?;
    options.section = Some(section_name.to_string());
    options.count = Some(1);
    options.age = Some(Age::Newest);
    options.sort = Some(SortOrder::Desc);

    let filtered = filter_entries(all_entries, &options);

    let template_name = self.display.template.as_deref().unwrap_or("last");
    let render_options = RenderOptions::from_config(template_name, &ctx.config);
    let output = format_items(&filtered, &render_options, &ctx.config, self.display.totals);

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use chrono::{Local, TimeZone};

  use super::*;
  use crate::taskpaper::{Document, Entry, Note, Section, Tag, Tags};

  fn default_cmd() -> Command {
    Command {
      display: DisplayArgs::default(),
      filter: FilterArgs::default(),
      pager: false,
    }
  }

  fn sample_ctx() -> AppContext {
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 14, 0, 0).unwrap(),
      "Working on project",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 3, 17, 15, 0, 0).unwrap(),
      "Meeting with team",
      Tags::from_iter(vec![Tag::new("meeting", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    AppContext {
      config: crate::config::Config::default(),
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
    }
  }

  mod call {
    use super::*;

    #[test]
    fn it_returns_ok() {
      let mut ctx = sample_ctx();
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_filters_by_section() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          section: Some("Currently".into()),
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_filters_by_tag() {
      let mut ctx = sample_ctx();
      let cmd = Command {
        filter: FilterArgs {
          tag: vec!["coding".into()],
          ..FilterArgs::default()
        },
        ..default_cmd()
      };

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }

    #[test]
    fn it_handles_empty_document() {
      let mut ctx = AppContext {
        config: crate::config::Config::default(),
        document: Document::new(),
        doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
      };
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
