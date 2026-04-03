use doing_ops::filter::{Age, filter_entries};
use doing_taskpaper::Entry;

use crate::{
  Result,
  cli::{AppContext, args::FilterArgs},
};

/// Tracks an entry's ID and section for locating it in the document.
#[derive(Clone, Debug)]
pub struct EntryLocation {
  pub id: String,
  pub section: String,
}

impl EntryLocation {
  /// Create an `EntryLocation` from an entry.
  pub fn from_entry(entry: &Entry) -> Self {
    Self {
      id: entry.id().to_string(),
      section: entry.section().to_string(),
    }
  }
}

/// Locate entries matching filters, or fall back to the last N entries from the section.
///
/// When `unfinished` is `true`, only entries without a `@done` tag are considered
/// in the no-filter fallback path and in filter options.
pub fn find_entries(
  filter: &FilterArgs,
  count: Option<usize>,
  unfinished: bool,
  ctx: &AppContext,
) -> Result<Vec<EntryLocation>> {
  let section = filter
    .section
    .clone()
    .unwrap_or_else(|| ctx.config.current_section.clone());

  let has_filters = !filter.tag.is_empty() || filter.search.is_some() || !filter.val.is_empty();

  if has_filters {
    let all_entries: Vec<Entry> = ctx.document.all_entries().into_iter().cloned().collect();

    let mut options = filter.to_filter_options(&ctx.config, ctx.include_notes)?;
    options.age = options.age.or(Some(Age::Newest));
    if count.is_some() {
      options.count = count;
    }
    options.unfinished = unfinished;

    let results = filter_entries(all_entries, &options);
    return Ok(results.iter().map(EntryLocation::from_entry).collect());
  }

  let count = count.unwrap_or(1);
  let entries = ctx.document.entries_in_section(&section);
  // Sort by date descending (with position as tiebreaker for same-timestamp
  // entries) so we always pick the newest entries regardless of file order.
  let mut candidates: Vec<(usize, &&Entry)> = entries
    .iter()
    .enumerate()
    .filter(|(_, e)| if unfinished { e.unfinished() } else { true })
    .collect();
  candidates.sort_by(|(i_a, a), (i_b, b)| b.date().cmp(&a.date()).then_with(|| i_b.cmp(i_a)));
  let locs: Vec<EntryLocation> = candidates
    .iter()
    .take(count)
    .map(|(_, e)| EntryLocation::from_entry(e))
    .collect();

  Ok(locs)
}

/// Look up a mutable reference to an entry by its location.
pub fn find_entry_mut<'a>(ctx: &'a mut AppContext, loc: &EntryLocation) -> Result<&'a mut Entry> {
  let section = ctx
    .document
    .section_by_name_mut(&loc.section)
    .ok_or_else(|| crate::cli::section_not_found_err(&loc.section))?;

  section
    .entries_mut()
    .iter_mut()
    .find(|e| e.id() == loc.id)
    .ok_or_else(|| crate::Error::Config("entry not found".into()))
}

/// Present an interactive selection prompt and return the chosen entries.
///
/// When `unfinished` is `true`, only entries without a `@done` tag are shown.
pub fn interactive_select(filter: &FilterArgs, unfinished: bool, ctx: &AppContext) -> Result<Vec<EntryLocation>> {
  let section = filter
    .section
    .clone()
    .unwrap_or_else(|| ctx.config.current_section.clone());

  let candidates: Vec<Entry> = ctx
    .document
    .entries_in_section(&section)
    .into_iter()
    .filter(|e| if unfinished { e.unfinished() } else { true })
    .cloned()
    .collect();

  if candidates.is_empty() {
    return Ok(vec![]);
  }

  let selected = crate::cli::interactive::select_entries(&candidates)?;
  Ok(selected.iter().map(EntryLocation::from_entry).collect())
}
