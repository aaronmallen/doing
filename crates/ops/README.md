# doing-ops

Domain operations for the [doing](https://github.com/aaronmallen/doing) CLI.

This crate provides the core business logic that sits between the CLI layer and the document model: filtering and
searching entries, automatic tagging, backup/undo management, tag queries, and note extraction.

## Modules

- `autotag` — automatic tag assignment based on config rules
- `backup` — atomic file writes with backup history
- `extract_note` — split entry text into title and note
- `filter` — entry filtering pipeline (tags, dates, search)
- `search` — fuzzy and pattern-based entry search
- `tag_filter` — tag-based entry filtering with wildcard support
- `tag_query` — structured tag query parsing and evaluation
- `undo` — undo/redo via backup file rotation

## License

MIT — see [LICENSE](../../LICENSE) for details.
