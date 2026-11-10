//! Template parsing and rendering for the doing CLI.
//!
//! Templates are format strings that control how entries are displayed. They
//! use `%`-prefixed tokens for placeholders and colors:
//!
//! ```text
//! %boldwhite%title%reset  %interval  %cyan[%section]%reset
//! ```
//!
//! # Placeholders
//!
//! `%title`, `%date`, `%shortdate`, `%section`, `%interval`, `%duration`,
//! `%note`, `%tags`, and more. Placeholders support optional width and
//! indentation modifiers (e.g. `%-10shortdate`, `%_2note`).
//!
//! # Colors
//!
//! Named ANSI colors (`%red`, `%boldcyan`, `%reset`), background variants
//! (`%bg_blue`), and hex colors (`%#ff8800`, `%bg#003366`).
//!
//! # Modules
//!
//! - [`colors`] — ANSI color name resolution and style application.
//! - [`parser`] — tokenizer that splits a template string into color spans
//!   and placeholders.
//! - [`renderer`] — applies parsed templates to entries, producing styled output.
//! - [`totals`] — per-tag duration totals appended to rendered output.
//! - [`wrap`] — word wrapping that respects ANSI escapes and tag values.

pub mod colors;
pub mod parser;
pub mod renderer;
pub mod totals;
pub mod wrap;
