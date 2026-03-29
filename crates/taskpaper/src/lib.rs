//! TaskPaper document parser and serializer for the doing CLI.
//!
//! This crate implements the document model that backs the doing file format.
//! A doing file is a subset of the [TaskPaper](https://www.taskpaper.com/) format:
//! sections (headers ending with `:`) contain entries (lines starting with `- `),
//! each with tags (`@name` or `@name(value)`) and optional indented notes.
//!
//! # Document model
//!
//! - [`Document`] — an ordered list of [`Section`]s with methods for querying,
//!   mutating, and persisting entries.
//! - [`Section`] — a named group of entries (e.g. "Currently", "Done").
//! - [`Entry`] — a single time-tracked item with a date, title, [`Tags`], and [`Note`].
//! - [`Tags`] — an ordered collection of [`Tag`] key-value pairs.
//!
//! # File I/O
//!
//! - [`create_file`] — create a new doing file with a default section.
//! - [`read_file`] — parse an on-disk file into a [`Document`].
//! - [`write_file`] — atomically write a [`Document`] back to disk.
//! - [`serialize`] — render a [`Document`] to its TaskPaper string representation.

pub mod document;
pub mod entries;
pub mod io;
mod note;
mod parser;
pub mod section;
pub mod serializer;
mod tags;

pub use document::Document;
pub use entries::Entry;
pub use io::{create_file, read_file, write_file};
pub use note::Note;
pub use parser::DEFAULT_SECTION;
pub use section::Section;
pub use serializer::serialize;
pub use tags::{Tag, Tags};
