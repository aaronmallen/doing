use chrono::{DateTime, Local};
use clap::{Args, ValueEnum};
use doing_config::{Config, SortOrder};
use doing_ops::{
  filter::{Age, FilterOptions},
  search,
  tag_filter::{BooleanMode, TagFilter},
  tag_query::TagQuery,
};
use doing_plugins::default_registry;
use doing_taskpaper::Entry;
use doing_template::{
  renderer::{RenderOptions, format_items_with_tag_sort},
  totals::{TagSortField, TagSortOrder, TotalsGrouping, TotalsOptions},
};
use doing_time::{DurationFormat, chronify, parse_range};

use crate::Result;

/// Which end of the chronological list to keep.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum AgeArg {
  /// Keep the most recent entries.
  Newest,
  /// Keep the oldest entries.
  Oldest,
}

impl From<AgeArg> for Age {
  fn from(value: AgeArg) -> Self {
    match value {
      AgeArg::Newest => Self::Newest,
      AgeArg::Oldest => Self::Oldest,
    }
  }
}

/// How multiple tag conditions are combined.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BoolArg {
  /// All specified tags must be present.
  And,
  /// None of the specified tags may be present.
  Not,
  /// At least one specified tag must be present.
  Or,
  /// Each tag carries its own +/- prefix (default).
  Pattern,
}

impl From<BoolArg> for BooleanMode {
  fn from(value: BoolArg) -> Self {
    match value {
      BoolArg::And => Self::And,
      BoolArg::Not => Self::Not,
      BoolArg::Or => Self::Or,
      BoolArg::Pattern => Self::Pattern,
    }
  }
}

/// How to group totals (tags or section).
#[derive(Clone, Debug, ValueEnum)]
pub enum TotalsGroupingArg {
  /// Group by section name.
  #[value(alias = "project", alias = "p")]
  Section,
  /// Group by tag (default).
  #[value(alias = "tag")]
  Tags,
}

/// Shared display/output arguments reused across commands.
#[derive(Args, Clone, Debug, Default)]
pub struct DisplayArgs {
  /// Grouping for totals display (tags, section)
  #[arg(long = "by", value_enum)]
  pub by: Vec<TotalsGroupingArg>,

  /// Named template from config to use for output
  #[arg(long, alias = "config_template")]
  pub config_template: Option<String>,

  /// Show elapsed time on open entries
  #[arg(long)]
  pub duration: bool,

  /// Output format
  #[arg(short, long)]
  pub output: Option<String>,

  /// Save the current options as a named view
  #[arg(long)]
  pub save: Option<String>,

  /// Sort order for results
  #[arg(long, value_enum)]
  pub sort: Option<SortArg>,

  /// Sort order for tag totals (asc/desc)
  #[arg(long, alias = "tag_order", value_enum)]
  pub tag_order: Option<SortArg>,

  /// Sort field for tag totals
  #[arg(long, alias = "tag_sort", value_enum)]
  pub tag_sort: Option<TagSortArg>,

  /// Inline template string for output (e.g. "%title", "%date - %title")
  #[arg(long)]
  pub template: Option<String>,

  /// Show time intervals on @done tasks
  #[arg(short = 't', long)]
  pub times: bool,

  #[arg(long = "no-times", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "times")]
  pub no_times: bool,

  /// Show section title in output; accepts an optional custom title string
  #[arg(long, num_args = 0..=1, default_missing_value = "")]
  pub title: Option<String>,

  /// Show tag time totals
  #[arg(long)]
  pub totals: bool,

  /// Format for totals display (clock, hm, natural, text, dhm, m)
  #[arg(long = "totals-format", alias = "totals_format")]
  pub totals_format: Option<String>,
}

impl DisplayArgs {
  /// Render entries using either an export plugin or the template pipeline.
  ///
  /// If `--output` matches a registered export plugin trigger, the plugin renders
  /// the entries directly. Otherwise, the standard template rendering is used.
  ///
  /// Returns an error if `--output` is specified but does not match any registered
  /// export plugin.
  pub fn render_entries(
    &self,
    entries: &[Entry],
    config: &Config,
    default_template: &str,
    include_notes: bool,
  ) -> Result<String> {
    // --config-template: always looks up a named template from config
    // --template: if it contains %, treat as inline template string; otherwise as config name
    if let Some(ref name) = self.config_template
      && !config.templates.contains_key(name.as_str())
    {
      return Err(crate::Error::Config(format!("template \"{name}\" not found in config")));
    }

    let template_name = self.config_template.as_deref().or(self.template.as_deref());
    let resolved_name = template_name.unwrap_or(default_template);
    let mut render_options = if let Some(inline_template) = self.template.as_ref().filter(|t| t.contains('%')) {
      let mut opts = RenderOptions::from_config(default_template, config);
      opts.template = inline_template.clone();
      opts
    } else {
      RenderOptions::from_config(resolved_name, config)
    };

    render_options.include_notes = include_notes;

    if let Some(ref format) = self.output {
      let registry = default_registry();
      if let Some(plugin) = registry.resolve(format) {
        return Ok(plugin.render(entries, &render_options, config));
      }
      let valid = registry.available_formats().join(", ");
      return Err(crate::Error::Plugin(format!(
        "\"{format}\" is not a recognized output format. Valid formats: {valid}"
      )));
    }

    let tag_sort_field = match self.tag_sort.unwrap_or_default() {
      TagSortArg::Name => TagSortField::Name,
      TagSortArg::Time => TagSortField::Time,
    };
    let tag_sort_order = match self.tag_order {
      Some(SortArg::Desc) => TagSortOrder::Desc,
      _ => TagSortOrder::Asc,
    };

    let totals_format_str = self.totals_format.as_deref().or_else(|| {
      let v = config.totals_format.as_str();
      if v.is_empty() { None } else { Some(v) }
    });

    let show_averages = totals_format_str.is_some_and(|s| s.eq_ignore_ascii_case("averages"));
    let totals_format = totals_format_str
      .filter(|s| !s.eq_ignore_ascii_case("averages"))
      .map(DurationFormat::from_config);

    let groupings: Vec<TotalsGrouping> = self
      .by
      .iter()
      .map(|g| match g {
        TotalsGroupingArg::Section => TotalsGrouping::Section,
        TotalsGroupingArg::Tags => TotalsGrouping::Tags,
      })
      .collect();

    Ok(format_items_with_tag_sort(
      entries,
      &render_options,
      config,
      self.title.as_deref(),
      TotalsOptions {
        duration_format: totals_format,
        enabled: self.totals,
        groupings,
        show_averages,
        sort_field: tag_sort_field,
        sort_order: tag_sort_order,
      },
    ))
  }
}

/// Shared filter arguments reused across commands.
#[derive(Args, Clone, Debug, Default)]
pub struct FilterArgs {
  /// Date range (e.g. "monday to friday")
  #[arg(long)]
  pub after: Option<String>,

  /// Which end to keep when limiting by count (newest/oldest)
  #[arg(long, value_enum)]
  pub age: Option<AgeArg>,

  /// Upper bound for entry date
  #[arg(long)]
  pub before: Option<String>,

  /// Boolean operator for combining tag filters
  #[arg(long = "bool", value_enum, ignore_case = true)]
  pub bool_op: Option<BoolArg>,

  /// Case sensitivity for search (smart/sensitive/ignore)
  #[arg(long)]
  pub case: Option<String>,

  /// Use exact (literal substring) matching for search
  #[arg(short = 'x', long)]
  pub exact: bool,

  /// Date range expression (e.g. "monday to friday")
  #[arg(long)]
  pub from: Option<String>,

  /// Negate all filter results
  #[arg(long)]
  pub not: bool,

  /// Only include entries with a recorded time interval
  #[arg(long)]
  pub only_timed: bool,

  /// Text search query
  #[arg(long)]
  pub search: Option<String>,

  /// Section name to filter by
  #[arg(short, long)]
  pub section: Option<String>,

  /// Tags to filter by (can be repeated)
  #[arg(long)]
  pub tag: Vec<String>,

  /// Only include unfinished entries (no @done tag)
  #[arg(short = 'u', long)]
  pub unfinished: bool,

  /// Tag value queries (e.g. "progress > 50")
  #[arg(long)]
  pub val: Vec<String>,
}

impl FilterArgs {
  /// Convert CLI filter arguments into a [`FilterOptions`] for the filter pipeline.
  pub fn to_filter_options(&self, config: &Config, include_notes: bool) -> Result<FilterOptions> {
    let after = self.resolve_after()?;
    let before = self.resolve_before()?;
    let (from_after, from_before) = self.resolve_from()?;

    let effective_after = from_after.or(after);
    let effective_before = from_before.or(before);

    let mut search_config = config.search.clone();
    if let Some(ref case_override) = self.case {
      search_config.case = case_override.clone();
    }
    if self.exact {
      search_config.matching = "exact".into();
    }

    let search = self
      .search
      .as_deref()
      .and_then(|q| search::parse_query(q, &search_config));

    let expanded_tags = expand_tags(&self.tag);

    let tag_filter = if expanded_tags.is_empty() {
      None
    } else {
      let mode = self.bool_op.map(BooleanMode::from).unwrap_or_default();
      Some(TagFilter::new(&expanded_tags, mode))
    };

    let tag_queries = self
      .val
      .iter()
      .map(|v| {
        if let Some(q) = TagQuery::parse(v) {
          Ok(q)
        } else if !expanded_tags.is_empty() {
          // Bare value: treat as equality check against the first --tag
          let tag_name = &expanded_tags[0];
          TagQuery::parse(&format!("{tag_name} == {v}"))
            .ok_or_else(|| crate::Error::Parse(format!("invalid tag query: {v}")))
        } else {
          Err(crate::Error::Parse(format!("invalid tag query: {v}")))
        }
      })
      .collect::<Result<Vec<_>>>()?;

    Ok(FilterOptions {
      after: effective_after,
      age: self.age.map(Age::from),
      before: effective_before,
      count: None,
      include_notes,
      negate: self.not,
      only_timed: self.only_timed,
      search,
      section: self.section.clone(),
      sort: None,
      tag_filter,
      tag_queries,
      unfinished: self.unfinished,
    })
  }

  fn resolve_after(&self) -> Result<Option<DateTime<Local>>> {
    self.after.as_deref().map(chronify).transpose()
  }

  fn resolve_before(&self) -> Result<Option<DateTime<Local>>> {
    self.before.as_deref().map(chronify).transpose()
  }

  fn resolve_from(&self) -> Result<DateRange> {
    match self.from.as_deref() {
      Some(expr) => {
        let (start, end) = parse_range(expr)?;
        Ok((Some(start), Some(end)))
      }
      None => Ok((None, None)),
    }
  }
}

/// Shared positional-count / flag-count pattern used by finish and cancel.
///
/// Provides a positional `COUNT` argument and a `--count` flag, with the positional
/// taking precedence.
#[derive(Args, Clone, Debug)]
pub struct CountArgs {
  /// Number of entries (positional)
  #[arg(index = 1, value_name = "COUNT")]
  pub count_pos: Option<usize>,

  /// Number of entries (flag)
  #[arg(short, long, default_value_t = 1)]
  pub count: usize,
}

impl CountArgs {
  /// Return the effective count, preferring the positional argument over the flag.
  pub fn effective_count(&self) -> usize {
    self.count_pos.unwrap_or(self.count)
  }
}

/// Expand comma-separated tags into individual tag names, trimming whitespace and removing empties.
pub fn expand_tags(tags: &[String]) -> Vec<String> {
  tags
    .iter()
    .flat_map(|t| t.split(',').map(|s| s.trim().to_string()))
    .filter(|s| !s.is_empty())
    .collect()
}

/// Sort direction for output.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SortArg {
  /// Sort in ascending order.
  Asc,
  /// Sort in descending order.
  Desc,
}

impl From<SortArg> for SortOrder {
  fn from(value: SortArg) -> Self {
    match value {
      SortArg::Asc => Self::Asc,
      SortArg::Desc => Self::Desc,
    }
  }
}

/// How tags are sorted in the totals section.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum TagSortArg {
  /// Sort tags alphabetically by name.
  #[default]
  Name,
  /// Sort tags by total time.
  Time,
}

type DateRange = (Option<DateTime<Local>>, Option<DateTime<Local>>);

#[cfg(test)]
mod test {
  use super::*;

  mod age_arg {
    use super::*;

    #[test]
    fn it_converts_to_age() {
      assert_eq!(Age::from(AgeArg::Newest), Age::Newest);
      assert_eq!(Age::from(AgeArg::Oldest), Age::Oldest);
    }
  }

  mod bool_arg {
    use super::*;

    #[test]
    fn it_converts_to_boolean_mode() {
      assert_eq!(BooleanMode::from(BoolArg::And), BooleanMode::And);
      assert_eq!(BooleanMode::from(BoolArg::Not), BooleanMode::Not);
      assert_eq!(BooleanMode::from(BoolArg::Or), BooleanMode::Or);
      assert_eq!(BooleanMode::from(BoolArg::Pattern), BooleanMode::Pattern);
    }
  }

  mod display_args {
    use super::*;

    mod render_entries {
      use super::*;

      #[test]
      fn it_renders_with_recognized_output_format() {
        let args = DisplayArgs {
          output: Some("json".into()),
          ..DisplayArgs::default()
        };
        let config = Config::default();

        let result = args.render_entries(&[], &config, "default", true);

        assert!(result.is_ok());
      }

      #[test]
      fn it_renders_with_template_when_no_output_specified() {
        let args = DisplayArgs::default();
        let config = Config::default();

        let result = args.render_entries(&[], &config, "default", true);

        assert!(result.is_ok());
      }

      #[test]
      fn it_returns_error_for_unrecognized_output_format() {
        let args = DisplayArgs {
          output: Some("falafel".into()),
          ..DisplayArgs::default()
        };
        let config = Config::default();

        let result = args.render_entries(&[], &config, "default", true);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
          err.contains("falafel"),
          "error should mention the invalid format: {err}"
        );
        assert!(err.contains("csv"), "error should list valid formats: {err}");
      }
    }
  }

  mod filter_args {
    use super::*;

    #[test]
    fn it_builds_tag_filter_from_comma_separated_tags() {
      let args = FilterArgs {
        tag: vec!["rust,code".into()],
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert!(options.tag_filter.is_some());
    }

    #[test]
    fn it_builds_tag_filter_with_default_bool() {
      let args = FilterArgs {
        tag: vec!["rust".into(), "code".into()],
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert!(options.tag_filter.is_some());
    }

    #[test]
    fn it_converts_default_args_to_default_options() {
      let args = FilterArgs::default();
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert!(options.after.is_none());
      assert!(options.age.is_none());
      assert!(options.before.is_none());
      assert!(options.count.is_none());
      assert!(options.include_notes);
      assert!(!options.negate);
      assert!(!options.only_timed);
      assert!(options.search.is_none());
      assert!(options.section.is_none());
      assert!(options.sort.is_none());
      assert!(options.tag_filter.is_none());
      assert!(options.tag_queries.is_empty());
      assert!(!options.unfinished);
    }

    #[test]
    fn it_parses_after_date() {
      let args = FilterArgs {
        after: Some("2024-01-15 10:00".into()),
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert!(options.after.is_some());
    }

    #[test]
    fn it_parses_before_date() {
      let args = FilterArgs {
        before: Some("2024-03-15 10:00".into()),
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert!(options.before.is_some());
    }

    #[test]
    fn it_parses_tag_queries() {
      let args = FilterArgs {
        val: vec!["progress > 50".into()],
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert_eq!(options.tag_queries.len(), 1);
    }

    #[test]
    fn it_passes_section_through() {
      let args = FilterArgs {
        section: Some("Archive".into()),
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert_eq!(options.section.as_deref(), Some("Archive"));
    }

    #[test]
    fn it_rejects_invalid_tag_queries() {
      let args = FilterArgs {
        val: vec!["not a valid query!!!".into()],
        ..Default::default()
      };
      let config = Config::default();

      assert!(args.to_filter_options(&config, true).is_err());
    }

    #[test]
    fn it_sets_negate_from_not_flag() {
      let args = FilterArgs {
        not: true,
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert!(options.negate);
    }

    #[test]
    fn it_sets_only_timed() {
      let args = FilterArgs {
        only_timed: true,
        ..Default::default()
      };
      let config = Config::default();

      let options = args.to_filter_options(&config, true).unwrap();

      assert!(options.only_timed);
    }
  }

  mod sort_arg {
    use super::*;

    #[test]
    fn it_converts_to_sort_order() {
      assert_eq!(SortOrder::from(SortArg::Asc), SortOrder::Asc);
      assert_eq!(SortOrder::from(SortArg::Desc), SortOrder::Desc);
    }
  }
}
