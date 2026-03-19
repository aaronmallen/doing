use chrono::{Duration, Local, NaiveTime};
use clap::Args;

use crate::{
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    pager,
  },
  config::SortOrder,
  errors::Result,
  ops::filter::filter_entries,
  template::renderer::{RenderOptions, format_items},
};

/// Show entries from yesterday.
///
/// Displays all entries created yesterday — ideal for standup reports.
/// Use `--after` or `--before` to narrow the time window within the day.
///
/// # Examples
///
/// ```text
/// doing yesterday               # all entries from yesterday
/// doing yesterday -S Currently  # yesterday's entries from "Currently"
/// doing yesterday --tag meeting # yesterday's entries tagged @meeting
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

    let mut options = self
      .filter
      .clone()
      .into_filter_options(&ctx.config, ctx.include_notes)?;
    options.section = Some(section_name.to_string());

    let yesterday = (Local::now() - Duration::days(1)).date_naive();
    if options.after.is_none() {
      options.after = yesterday.and_time(NaiveTime::MIN).and_local_timezone(Local).single();
    }
    if options.before.is_none() {
      options.before = yesterday
        .and_hms_opt(23, 59, 59)
        .and_then(|dt| dt.and_local_timezone(Local).single());
    }

    let sort_order = self.display.sort.map(SortOrder::from).or(Some(ctx.config.order));
    options.sort = sort_order;

    let filtered = filter_entries(all_entries, &options);

    let template_name = self.display.template.as_deref().unwrap_or("yesterday");
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
  use chrono::{Duration, Local, TimeZone};

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
    let yesterday = Local::now() - Duration::days(1);
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      yesterday,
      "Yesterday's work",
      Tags::from_iter(vec![Tag::new("coding", None::<String>)]),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    section.add_entry(Entry::new(
      Local.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
      "Old entry",
      Tags::new(),
      Note::new(),
      "Currently",
      None::<String>,
    ));
    doc.add_section(section);

    AppContext {
      config: crate::config::Config::default(),
      default_answer: false,
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
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
    fn it_handles_empty_document() {
      let mut ctx = AppContext {
        config: crate::config::Config::default(),
        default_answer: false,
        document: Document::new(),
        doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
        include_notes: true,
        no: false,
        noauto: false,
        stdout: false,
        use_color: false,
        use_pager: false,
        yes: false,
      };
      let cmd = default_cmd();

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
