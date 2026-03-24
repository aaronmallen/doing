//! A command-line tool for remembering what you were doing and tracking what
//! you've done.
//!
//! `doing` is a Rust rewrite of Brett Terpstra's
//! [doing](https://github.com/ttscoff/doing) time-tracking CLI. It stores
//! entries in a TaskPaper-formatted file organized by sections, with support
//! for tags, notes, natural-language time expressions, and multiple export
//! formats.
//!
//! # Workspace crates
//!
//! The application is split into focused library crates:
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | [`doing_error`] | Shared [`Error`] enum and [`Result`] alias |
//! | [`doing_time`] | Natural-language time parsing and duration formatting |
//! | [`doing_taskpaper`] | TaskPaper document model, parser, and serializer |
//! | [`doing_config`] | Multi-format config loading with env-var overrides |
//! | [`doing_ops`] | Domain operations (filter, search, autotag, backup, undo) |
//! | [`doing_template`] | `%`-token template language for entry rendering |
//! | [`doing_plugins`] | Export/import plugin registry and built-in formats |
//!
//! This crate provides the CLI layer: argument parsing, subcommand dispatch,
//! editor integration, interactive prompts, and paged output.

pub mod cli;

pub use doing_error::{Error, Result};
