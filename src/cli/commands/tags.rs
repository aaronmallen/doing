use std::collections::HashMap;

use clap::{Args, ValueEnum};
use doing_ops::filter::filter_entries;

use crate::{
  Result,
  cli::{AppContext, args::FilterArgs},
};

/// List all tags in the doing file with optional counts and sorting.
///
/// Scans entries across all sections (or a specific section) and collects
/// every unique tag with its usage count. Tags are compared case-insensitively;
/// the first-seen casing is preserved in output.
///
/// # Examples
///
/// ```text
/// doing tags                        # list all tags
/// doing tags --counts               # show usage counts
/// doing tags -s Currently           # tags from one section
/// doing tags --sort name --order asc
/// doing tags --line                 # single-line output for scripting
/// doing tags 10                     # limit to top 10 tags
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Show usage counts alongside tag names
  #[arg(short, long, overrides_with = "no_counts")]
  counts: bool,

  #[arg(long = "no-counts", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "counts")]
  no_counts: bool,

  #[command(flatten)]
  filter: FilterArgs,

  /// Output tags on a single line (for scripting)
  #[arg(short = 'l', long, overrides_with = "no_line")]
  line: bool,

  #[arg(long = "no-line", action = clap::ArgAction::SetTrue, hide = true, overrides_with = "line")]
  no_line: bool,

  /// Maximum number of tags to display
  #[arg(index = 1, value_name = "MAX_COUNT")]
  max_count: Option<usize>,

  /// Sort order
  #[arg(short = 'o', long, default_value = "asc")]
  order: OrderArg,

  /// Sort by name, count, or time (total interval)
  #[arg(short = 'S', long, default_value = "name")]
  sort: SortArg,
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

    let filter_options = self.filter.clone().into_filter_options(&ctx.config, false)?;
    let filtered = filter_entries(all_entries, &filter_options);
    let entry_refs: Vec<_> = filtered.iter().collect();

    let mut tag_counts = collect_tags(&entry_refs);

    match self.sort {
      SortArg::Count => sort_by_count(&mut tag_counts, &self.order),
      SortArg::Name => sort_by_name(&mut tag_counts, &self.order),
      SortArg::Time => sort_by_time(&mut tag_counts, &entry_refs, &self.order),
    }

    if let Some(max) = self.max_count {
      tag_counts.truncate(max);
    }

    if tag_counts.is_empty() {
      return Ok(());
    }

    if self.line && !self.no_line {
      let names: Vec<String> = tag_counts.iter().map(|(name, _)| format!("@{name}")).collect();
      println!("{}", names.join(" "));
    } else {
      for (name, count) in &tag_counts {
        if self.counts && !self.no_counts {
          println!("{name} ({count})");
        } else {
          println!("{name}");
        }
      }
    }

    Ok(())
  }
}

/// Sort direction for the tags command.
#[derive(Clone, Debug, ValueEnum)]
enum OrderArg {
  /// Ascending order
  Asc,
  /// Descending order
  Desc,
}

/// Sort field for the tags command.
#[derive(Clone, Debug, ValueEnum)]
enum SortArg {
  /// Sort by usage count
  Count,
  /// Sort alphabetically by tag name
  Name,
  /// Sort by total time interval
  Time,
}

fn collect_tags(entries: &[&doing_taskpaper::Entry]) -> Vec<(String, usize)> {
  let mut counts: HashMap<String, (String, usize)> = HashMap::new();

  for entry in entries {
    for tag in entry.tags().iter() {
      let key = tag.name().to_ascii_lowercase();
      counts
        .entry(key)
        .and_modify(|(_, count)| *count += 1)
        .or_insert_with(|| (tag.name().to_string(), 1));
    }
  }

  counts.into_values().collect()
}

fn sort_by_count(tag_counts: &mut [(String, usize)], order: &OrderArg) {
  tag_counts.sort_by(|(a_name, a_count), (b_name, b_count)| {
    let cmp = a_count
      .cmp(b_count)
      .then_with(|| a_name.to_ascii_lowercase().cmp(&b_name.to_ascii_lowercase()));
    match order {
      OrderArg::Asc => cmp,
      OrderArg::Desc => cmp.reverse(),
    }
  });
}

fn sort_by_name(tag_counts: &mut [(String, usize)], order: &OrderArg) {
  tag_counts.sort_by(|(a, _), (b, _)| {
    let cmp = a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase());
    match order {
      OrderArg::Asc => cmp,
      OrderArg::Desc => cmp.reverse(),
    }
  });
}

fn sort_by_time(tag_counts: &mut [(String, usize)], entries: &[&doing_taskpaper::Entry], order: &OrderArg) {
  let mut tag_durations: HashMap<String, i64> = HashMap::new();

  for entry in entries {
    if let Some(interval) = entry.interval() {
      for tag in entry.tags().iter() {
        let key = tag.name().to_ascii_lowercase();
        *tag_durations.entry(key).or_default() += interval.num_seconds();
      }
    }
  }

  tag_counts.sort_by(|(a, _), (b, _)| {
    let da = tag_durations.get(&a.to_ascii_lowercase()).copied().unwrap_or(0);
    let db = tag_durations.get(&b.to_ascii_lowercase()).copied().unwrap_or(0);
    let cmp = da.cmp(&db);
    match order {
      OrderArg::Asc => cmp,
      OrderArg::Desc => cmp.reverse(),
    }
  });
}

#[cfg(test)]
mod test {
  use chrono::{Duration, Local};
  use doing_taskpaper::{Entry, Note, Tag, Tags};

  use super::*;

  fn make_entry(title: &str, tag_names: &[&str]) -> Entry {
    let mut tags = Tags::new();
    for name in tag_names {
      tags.add(Tag::new(*name, None::<String>));
    }
    Entry::new(Local::now(), title, tags, Note::new(), "Currently", None::<String>)
  }

  fn make_timed_entry(title: &str, tag_names: &[&str], minutes: i64) -> Entry {
    let now = Local::now();
    let start = now - Duration::minutes(minutes);
    let mut tags = Tags::new();
    for name in tag_names {
      tags.add(Tag::new(*name, None::<String>));
    }
    tags.add(Tag::new(
      "done",
      Some(now.format(crate::cli::DONE_DATE_FORMAT).to_string()),
    ));
    Entry::new(start, title, tags, Note::new(), "Currently", None::<String>)
  }

  mod collect_tags {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_counts_tag_occurrences() {
      let e1 = make_entry("Task A", &["coding", "rust"]);
      let e2 = make_entry("Task B", &["coding"]);
      let entries: Vec<&Entry> = vec![&e1, &e2];

      let mut result = collect_tags(&entries);
      result.sort_by(|(a, _), (b, _)| a.cmp(b));

      assert_eq!(result.len(), 2);
      assert_eq!(result[0], ("coding".to_string(), 2));
      assert_eq!(result[1], ("rust".to_string(), 1));
    }

    #[test]
    fn it_deduplicates_case_insensitively() {
      let e1 = make_entry("Task A", &["Coding"]);
      let e2 = make_entry("Task B", &["coding"]);
      let entries: Vec<&Entry> = vec![&e1, &e2];

      let result = collect_tags(&entries);

      assert_eq!(result.len(), 1);
      assert_eq!(result[0].1, 2);
    }

    #[test]
    fn it_returns_empty_for_no_tags() {
      let e1 = make_entry("Task A", &[]);
      let entries: Vec<&Entry> = vec![&e1];

      let result = collect_tags(&entries);

      assert!(result.is_empty());
    }
  }

  mod sort_by_count {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_breaks_ties_by_name() {
      let mut tags = vec![("zebra".to_string(), 2), ("alpha".to_string(), 2)];

      sort_by_count(&mut tags, &OrderArg::Asc);

      assert_eq!(tags[0].0, "alpha");
      assert_eq!(tags[1].0, "zebra");
    }

    #[test]
    fn it_sorts_ascending_by_count() {
      let mut tags = vec![
        ("rust".to_string(), 3),
        ("coding".to_string(), 1),
        ("review".to_string(), 2),
      ];

      sort_by_count(&mut tags, &OrderArg::Asc);

      assert_eq!(tags[0].0, "coding");
      assert_eq!(tags[1].0, "review");
      assert_eq!(tags[2].0, "rust");
    }

    #[test]
    fn it_sorts_descending_by_count() {
      let mut tags = vec![
        ("coding".to_string(), 1),
        ("rust".to_string(), 3),
        ("review".to_string(), 2),
      ];

      sort_by_count(&mut tags, &OrderArg::Desc);

      assert_eq!(tags[0].0, "rust");
      assert_eq!(tags[1].0, "review");
      assert_eq!(tags[2].0, "coding");
    }
  }

  mod sort_by_name {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_sorts_ascending() {
      let mut tags = vec![("rust".to_string(), 1), ("coding".to_string(), 2)];

      sort_by_name(&mut tags, &OrderArg::Asc);

      assert_eq!(tags[0].0, "coding");
      assert_eq!(tags[1].0, "rust");
    }

    #[test]
    fn it_sorts_descending() {
      let mut tags = vec![("coding".to_string(), 2), ("rust".to_string(), 1)];

      sort_by_name(&mut tags, &OrderArg::Desc);

      assert_eq!(tags[0].0, "rust");
      assert_eq!(tags[1].0, "coding");
    }
  }

  mod sort_by_time {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_sorts_by_total_interval() {
      let e1 = make_timed_entry("Task A", &["coding"], 60);
      let e2 = make_timed_entry("Task B", &["rust"], 120);
      let entries: Vec<&Entry> = vec![&e1, &e2];

      let mut tags = vec![("coding".to_string(), 1), ("rust".to_string(), 1)];

      sort_by_time(&mut tags, &entries, &OrderArg::Asc);

      assert_eq!(tags[0].0, "coding");
      assert_eq!(tags[1].0, "rust");
    }

    #[test]
    fn it_sorts_descending_by_time() {
      let e1 = make_timed_entry("Task A", &["coding"], 60);
      let e2 = make_timed_entry("Task B", &["rust"], 120);
      let entries: Vec<&Entry> = vec![&e1, &e2];

      let mut tags = vec![("coding".to_string(), 1), ("rust".to_string(), 1)];

      sort_by_time(&mut tags, &entries, &OrderArg::Desc);

      assert_eq!(tags[0].0, "rust");
      assert_eq!(tags[1].0, "coding");
    }
  }
}
