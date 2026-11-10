//! Natural-language time parsing and duration formatting for the doing CLI.
//!
//! This crate converts human-friendly time expressions into concrete
//! `chrono::DateTime` and `chrono::Duration` values, and formats durations
//! back into display strings.
//!
//! # Key entry points
//!
//! - [`chronify`] — parse expressions like `"yesterday 3pm"`, `"2 hours ago"`,
//!   or `"last monday"` into a `DateTime<Local>`.
//! - [`parse_range`] — parse range expressions like `"monday to friday"` into
//!   a `(DateTime<Local>, DateTime<Local>)` tuple.
//! - [`parse_duration`] — parse duration strings like `"1h30m"`, `"90"`, or
//!   `"1 hour 30 minutes"` into a `chrono::Duration`.
//! - [`FormattedDuration`] — render a `Duration` in one of several display
//!   formats (clock, natural language, abbreviated, etc.).
//! - [`FormattedShortdate`] — render a `DateTime` as a relative or absolute
//!   short date string.

mod duration;
pub mod format;
pub mod parser;
pub mod range;

pub use duration::parse_duration;
pub use format::{DurationFormat, FormattedDuration, FormattedShortdate, ShortdateFormatConfig, format_tag_total};
pub use parser::chronify;
pub use range::parse_range;
