use chrono::{Local, NaiveTime};
use clap::Args;
use doing_config::SortOrder;

use crate::{
  Result,
  cli::{
    AppContext,
    args::{DisplayArgs, FilterArgs},
    pager,
  },
  ops::filter::filter_entries,
};

/// Show entries from today.
///
/// Displays all entries created today. Use `--after` or `--before` to
/// narrow the time window within the day.
///
/// # Examples
///
/// ```text
/// doing today                   # all entries from today
/// doing today -S Currently      # today's entries from "Currently"
/// doing today --tag meeting     # today's entries tagged @meeting
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

    if options.after.is_none() {
      let today = Local::now().date_naive();
      options.after = today.and_time(NaiveTime::MIN).and_local_timezone(Local).single();
    }

    let sort_order = self.display.sort.map(SortOrder::from).or(Some(ctx.config.order));
    options.sort = sort_order;

    let filtered = filter_entries(all_entries, &options);

    let output = self
      .display
      .render_entries(&filtered, &ctx.config, "today", ctx.include_notes)?;

    if !output.is_empty() {
      pager::output(&output, &ctx.config, self.pager || ctx.use_pager)?;
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
    let now = Local::now();
    let mut doc = Document::new();
    let mut section = Section::new("Currently");
    section.add_entry(Entry::new(
      now,
      "Working on project today",
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
      config: doing_config::Config::default(),
      default_answer: false,
      document: doc,
      doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
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
        config: doing_config::Config::default(),
        default_answer: false,
        document: Document::new(),
        doing_file: std::path::PathBuf::from("/tmp/test_doing.md"),
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

      let result = cmd.call(&mut ctx);

      assert!(result.is_ok());
    }
  }
}
