//! TaskPaper document parser and serializer for the doing CLI.

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
pub use section::Section;
pub use serializer::serialize;
pub use tags::{Tag, Tags};
