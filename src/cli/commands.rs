use doing_taskpaper::{Entry, Section, Tag};

use crate::cli::AppContext;

pub mod actions;
pub mod again;
pub mod archive;
pub mod autotag;
pub mod budget;
pub mod cancel;
pub mod changes;
pub mod choose;
pub mod colors;
#[allow(clippy::module_inception)]
pub mod commands;
pub mod commands_accepting;
pub mod completion;
pub mod config;
pub mod done;
pub mod finish;
pub mod grep;
pub mod import;
pub mod last;
pub mod mark;
pub mod meanwhile;
pub mod note;
pub mod now;
pub mod on;
pub mod open;
pub mod plugins;
pub mod recent;
pub mod redo;
pub mod reset;
pub mod rotate;
pub mod sections;
pub mod select;
pub mod show;
pub mod since;
pub mod tag;
pub mod tag_dir;
pub mod tags;
pub mod template;
pub mod today;
pub mod undo;
pub mod update;
pub mod view;
pub mod views;
pub mod yesterday;

/// Move entries matching `entry_ids` from `section_name` to the Archive section.
///
/// When `add_from_tag` is true, each moved entry gets a `@from(section_name)` tag
/// (used by finish). When false, entries are moved as-is (used by cancel).
pub fn archive_entries_by_id(
  ctx: &mut AppContext,
  section_name: &str,
  entry_ids: &[String],
  add_from_tag: bool,
) -> crate::Result<()> {
  if !ctx.document.has_section("Archive") {
    ctx.document.add_section(Section::new("Archive"));
  }

  let section = ctx
    .document
    .section_by_name_mut(section_name)
    .ok_or_else(|| crate::cli::section_not_found_err(section_name))?;

  let to_move: Vec<Entry> = section
    .entries_mut()
    .iter()
    .filter(|e| entry_ids.contains(&e.id().to_string()))
    .cloned()
    .collect();

  section
    .entries_mut()
    .retain(|e| !entry_ids.contains(&e.id().to_string()));

  let archive = ctx.document.section_by_name_mut("Archive").unwrap();
  for mut entry in to_move {
    if add_from_tag {
      entry.tags_mut().add(Tag::new("from", Some(section_name)));
    }
    archive.add_entry(entry);
  }

  Ok(())
}
