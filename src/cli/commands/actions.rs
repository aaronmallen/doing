use std::{fs, path::Path};

use doing_plugins::default_registry;
use doing_taskpaper::{Entry, Section, Tag};
use doing_template::renderer::{RenderOptions, format_items};

use crate::{
  Result,
  cli::{AppContext, pager},
};

pub fn action_archive(ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
  if !ctx.document.has_section("Archive") {
    ctx.document.add_section(Section::new("Archive"));
  }

  let ids: Vec<String> = selected.iter().map(|e| e.id().to_string()).collect();
  let sections: Vec<String> = selected.iter().map(|e| e.section().to_string()).collect();

  for entry in selected {
    ctx
      .document
      .section_by_name_mut("Archive")
      .unwrap()
      .add_entry(entry.clone());
  }

  for (id, section_name) in ids.iter().zip(sections.iter()) {
    if let Some(section) = ctx.document.section_by_name_mut(section_name) {
      section.remove_entry(id);
    }
  }

  let count = selected.len();
  if count == 1 {
    ctx.status("Archived 1 entry");
  } else {
    ctx.status(format!("Archived {count} entries"));
  }

  Ok(())
}

pub fn action_cancel(ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
  let mut count = 0;

  for entry in selected {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
      && e.unfinished()
      && e.should_finish(&ctx.config.never_finish)
    {
      e.tags_mut().add(Tag::new("done", None::<String>));
      count += 1;
    }
  }

  if selected.len() == 1 && count == 0 {
    ctx.status("Entry already finished or excluded by never_finish");
  } else if count == 1 {
    ctx.status("Cancelled 1 entry");
  } else {
    ctx.status(format!("Cancelled {count} entries"));
  }

  Ok(())
}

pub fn action_delete(ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
  for entry in selected {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section()) {
      section.remove_entry(entry.id());
    }
  }

  let count = selected.len();
  if count == 1 {
    ctx.status("Deleted 1 entry");
  } else {
    ctx.status(format!("Deleted {count} entries"));
  }

  Ok(())
}

pub fn action_finish(ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
  let now = chrono::Local::now();
  let mut count = 0;

  for entry in selected {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
      && e.unfinished()
      && e.should_finish(&ctx.config.never_finish)
    {
      let done_value = if e.should_time(&ctx.config.never_time) {
        Some(now.format(crate::cli::DONE_DATE_FORMAT).to_string())
      } else {
        None
      };
      e.tags_mut().add(Tag::new("done", done_value));
      count += 1;
    }
  }

  if selected.len() == 1 && count == 0 {
    ctx.status("Entry already finished or excluded by never_finish");
  } else if count == 1 {
    ctx.status("Marked 1 entry as @done");
  } else {
    ctx.status(format!("Marked {count} entries as @done"));
  }

  Ok(())
}

pub fn action_flag(ctx: &mut AppContext, selected: &[Entry]) -> Result<()> {
  let marker_tag = ctx.config.marker_tag.clone();
  let mut flagged = 0usize;
  let mut unflagged = 0usize;

  for entry in selected {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
    {
      if e.tags().has(&marker_tag) {
        e.tags_mut().remove(&marker_tag);
        unflagged += 1;
      } else {
        e.tags_mut().add(Tag::new(&marker_tag, None::<String>));
        flagged += 1;
      }
    }
  }

  let total = flagged + unflagged;
  if total == 1 {
    if flagged == 1 {
      ctx.status("Flagged 1 entry");
    } else {
      ctx.status("Unflagged 1 entry");
    }
  } else {
    ctx.status(format!("Flagged {flagged}, unflagged {unflagged} entries"));
  }

  Ok(())
}

pub fn action_move(ctx: &mut AppContext, selected: &[Entry], target: &str) -> Result<()> {
  if !ctx.document.has_section(target) {
    ctx.document.add_section(Section::new(target));
  }

  let ids: Vec<String> = selected.iter().map(|e| e.id().to_string()).collect();
  let sections: Vec<String> = selected.iter().map(|e| e.section().to_string()).collect();

  for entry in selected {
    ctx
      .document
      .section_by_name_mut(target)
      .unwrap()
      .add_entry(entry.clone());
  }

  for (id, section_name) in ids.iter().zip(sections.iter()) {
    if section_name != target
      && let Some(section) = ctx.document.section_by_name_mut(section_name)
    {
      section.remove_entry(id);
    }
  }

  let count = selected.len();
  if count == 1 {
    ctx.status(format!("Moved 1 entry to {target}"));
  } else {
    ctx.status(format!("Moved {count} entries to {target}"));
  }

  Ok(())
}

pub fn action_output(
  ctx: &AppContext,
  selected: &[Entry],
  output_format: Option<&str>,
  save_to: Option<&Path>,
) -> Result<()> {
  let mut render_options = RenderOptions::from_config("default", &ctx.config);
  render_options.include_notes = ctx.include_notes;
  let output = if let Some(format) = output_format {
    let registry = default_registry();
    if let Some(plugin) = registry.resolve(format) {
      plugin.render(selected, &render_options, &ctx.config)
    } else {
      format_items(selected, &render_options, &ctx.config, false)
    }
  } else {
    format_items(selected, &render_options, &ctx.config, false)
  };

  if let Some(path) = save_to {
    fs::write(path, &output)?;
    ctx.status(format!("Saved {} entries to {}", selected.len(), path.display()));
  } else {
    pager::output(&output, &ctx.config, ctx.use_pager)?;
  }

  Ok(())
}

pub fn action_tag(ctx: &mut AppContext, selected: &[Entry], tag_names: &[&str]) -> Result<()> {
  if tag_names.is_empty() {
    return Err(crate::Error::Config("no tags specified".into()));
  }

  for entry in selected {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
    {
      for name in tag_names {
        e.tags_mut().add(Tag::new(*name, None::<String>));
      }
    }
  }

  let count = selected.len();
  if count == 1 {
    ctx.status("Tagged 1 entry");
  } else {
    ctx.status(format!("Tagged {count} entries"));
  }

  Ok(())
}

pub fn action_remove_tag(ctx: &mut AppContext, selected: &[Entry], tag_names: &[&str]) -> Result<()> {
  if tag_names.is_empty() {
    return Err(crate::Error::Config("no tags specified".into()));
  }

  for entry in selected {
    if let Some(section) = ctx.document.section_by_name_mut(entry.section())
      && let Some(e) = section.entries_mut().iter_mut().find(|e| e.id() == entry.id())
    {
      for name in tag_names {
        e.tags_mut().remove(name);
      }
    }
  }

  let count = selected.len();
  if count == 1 {
    ctx.status("Removed tags from 1 entry");
  } else {
    ctx.status(format!("Removed tags from {count} entries"));
  }

  Ok(())
}
