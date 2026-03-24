//! Domain operations for the doing CLI.
//!
//! This crate contains the core business logic that sits between the CLI layer
//! and the data model. It operates on [`doing_taskpaper::Entry`] values and
//! [`doing_config::Config`] settings without any knowledge of terminal I/O.
//!
//! # Modules
//!
//! - [`autotag`] — automatic tag assignment based on title keywords, synonyms,
//!   and regex transforms.
//! - [`backup`] — timestamped backup creation with path-hash isolation so
//!   multiple doing files maintain independent histories.
//! - [`extract_note`] — split raw entry text into a title and an optional note.
//! - [`filter`] — composable entry filter pipeline (section, tags, search, date
//!   range, count limit, sort order).
//! - [`search`] — fuzzy, exact, pattern, and regex text matching against entries.
//! - [`tag_filter`] — wildcard-aware tag inclusion/exclusion filtering.
//! - [`tag_query`] — structured boolean query parser for tag expressions
//!   (e.g. `"@done and not @flagged"`).
//! - [`undo`] — undo/redo by rotating through backup history.

pub mod autotag;
pub mod backup;
pub mod extract_note;
pub mod filter;
pub mod search;
pub mod tag_filter;
pub mod tag_query;
pub mod undo;
