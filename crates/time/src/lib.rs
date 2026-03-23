//! doing-time crate.

mod duration;
pub mod format;
pub mod parser;
pub mod range;

pub use duration::parse_duration;
pub use format::{DurationFormat, FormattedDuration, FormattedShortdate, ShortdateFormatConfig, format_tag_total};
pub use parser::chronify;
pub use range::parse_range;
