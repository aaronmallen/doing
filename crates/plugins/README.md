# doing-plugins

Export and import plugins for the [doing](https://github.com/aaronmallen/doing) CLI.

This crate provides the plugin system for exporting entries to various formats and importing entries from external
sources.

## Export Plugins

- `byday` — group entries by day
- `csv` — comma-separated values
- `dayone` — Day One journal format
- `doing` — native doing file format
- `html` — HTML with CSS styling
- `json` — JSON export
- `markdown` — Markdown format
- `taskpaper` — TaskPaper format
- `timeline` — chronological timeline view

## Import Plugins

- `calendar` — iCalendar (ICS) files
- `doing` — doing file format
- `json` — doing JSON export format
- `timing` — Timing.app JSON reports

## License

MIT — see [LICENSE](../../LICENSE) for details.
