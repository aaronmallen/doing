use std::collections::HashMap;

use clap::{Args, ValueEnum};

use crate::{cli::AppContext, errors::Result};

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
/// ```
#[derive(Args, Clone, Debug)]
pub struct Command {
  /// Show usage counts alongside tag names
  #[arg(short = 'c', long)]
  counts: bool,

  /// Output tags on a single line (for scripting)
  #[arg(short = 'l', long)]
  line: bool,

  /// Sort order
  #[arg(long, default_value = "asc")]
  order: OrderArg,

  /// Count tags only within a specific section
  #[arg(short = 's', long)]
  section: Option<String>,

  /// Sort by name or time (total interval)
  #[arg(short = 'S', long, default_value = "name")]
  sort: SortArg,
}

/// Sort field for the tags command.
#[derive(Clone, Debug, ValueEnum)]
enum SortArg {
  /// Sort alphabetically by tag name
  Name,
  /// Sort by total time interval
  Time,
}

/// Sort direction for the tags command.
#[derive(Clone, Debug, ValueEnum)]
enum OrderArg {
  /// Ascending order
  Asc,
  /// Descending order
  Desc,
}

impl Command {
  pub fn call(&self, ctx: &mut AppContext) -> Result<()> {
    let entries = match &self.section {
      Some(name) => ctx.document.entries_in_section(name),
      None => ctx.document.all_entries(),
    };

    let mut tag_counts = collect_tags(&entries);

    match self.sort {
      SortArg::Name => sort_by_name(&mut tag_counts, &self.order),
      SortArg::Time => sort_by_time(&mut tag_counts, &entries, &self.order),
    }

    if tag_counts.is_empty() {
      println!("No tags found.");
      return Ok(());
    }

    if self.line {
      let names: Vec<&str> = tag_counts.iter().map(|(name, _)| name.as_str()).collect();
      println!("{}", names.join(" "));
    } else {
      for (name, count) in &tag_counts {
        if self.counts {
          println!("{name} ({count})");
        } else {
          println!("{name}");
        }
      }
    }

    Ok(())
  }
}

fn collect_tags(entries: &[&crate::taskpaper::Entry]) -> Vec<(String, usize)> {
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

fn sort_by_name(tag_counts: &mut [(String, usize)], order: &OrderArg) {
  tag_counts.sort_by(|(a, _), (b, _)| {
    let cmp = a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase());
    match order {
      OrderArg::Asc => cmp,
      OrderArg::Desc => cmp.reverse(),
    }
  });
}

fn sort_by_time(tag_counts: &mut [(String, usize)], entries: &[&crate::taskpaper::Entry], order: &OrderArg) {
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

  use super::*;
  use crate::taskpaper::{Entry, Note, Tag, Tags};

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
    tags.add(Tag::new("done", Some(now.format("%Y-%m-%d %H:%M").to_string())));
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
