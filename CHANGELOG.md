# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Break Versioning].

## [Unreleased]

## [v0.0.1-alpha.3] - 2026-03-20

### Added

- Shorthand duration expressions (`24h`, `30m`, `1d2h`) accepted anywhere natural language time is used (see [#14])
- `DOING_FILE` environment variable to specify the doing file path (see [#27])
- `--no-pager` global flag to disable pager output when `paginate` is enabled in config (see [#28])
- `-i`/`--interactive` flag on `again`, `cancel`, `finish`, `grep`, `mark`, `note`, `recent`,
  `reset`, `show`, and `tag` for entry selection menus (see [#30])
- `-u`/`--unfinished` flag on `cancel`, `finish`, `mark`, and `tag` to filter to entries without `@done` (see [#31])

### Fixed

- `show --times` displays time intervals on finished entries (see [#3])
- `show --section` respects the flag when positional argument is omitted (see [#4])
- `rotate` archive filename uses monthly (`YYYY-MM`) suffix instead of daily (see [#5])
- Sequential `doing undo` walks backwards through history instead of replaying the same backup (see [#6])
- `reset --from` filters by date range instead of overwriting start/done times (see [#7])
- Unrecognized `--output` format reports an error instead of silently falling back (see [#17])
- `grep` searches tag names in addition to titles and notes (see [#22])
- `--bool` flag accepts values case-insensitively (see [#25])
- `archive` adds `@from(SectionName)` tag by default (see [#36])
- `meanwhile` removes `@meanwhile` tag from the finished entry when replacing (see [#38])
- Default display template matches Ruby doing formatting (colors, duration format, layout) (see [#46])
- Status messages print clean text to stderr instead of structured log format (see [#48])
- `budget` argument order corrected to `TAG [AMOUNT]` (see [#52])
- Display output includes trailing newline (see [#63])
- `again` considers all entries regardless of `@done` status (see [#64])
- `--at` resolves bare times to today instead of yesterday (see [#66])
- `--bool` defaults to `pattern`; `+`/`-` tag prefixes work without explicit `--bool pattern` (see [#67])
- `done` with no arguments finds the last unfinished entry when the last entry is already `@done` (see [#68])
- `--noauto` short flag corrected to uppercase `-X` to avoid conflict with `-x`/`--exact` (see [#70])
- `on` command parses bare day-of-week names (e.g. `doing on friday`) (see [#71])
- `-t` short flag remapped to `--times` on display commands; `--tag` is long-only (see [#72])
- `--back` gains `--started`/`--since` aliases, `--at` gains `--finished`, `--took` gains `--for` (see [#79])
- Default display template width and alignment adjusted (see [#86])

## [v0.0.1-alpha.2] - 2026-03-20

### Fixed

- Default template includes colors (`%boldwhite`, `%boldcyan`, `%cyan`) matching Ruby doing style
- `archive` and `rotate` process all entries instead of only `@done` entries (see [#18])
- `again` selects last unfinished entry instead of last `@done` entry (see [#19])
- `tag`, `note`, `mark`, and `last` skip `@done` entries in default selection (see [#20])
- `reset` removes `@done` tag by default without requiring `--resume` (see [#21])
- Undo history isolated per doing file path (see [#23])
- `reset` no longer crashes with `--search`, `--back`, or `--from` when search config is unset (see [#24])
- Tags preserved in entry display output and info messages (see [#26])
- `-s` short flag remapped to `--section` instead of `--search` (see [#33])
- JSON output schema matches Ruby doing format (`id`, `done`, `end_date`, `timers` fields, timezone) (see [#44])
- `tag-dir` handles empty `.doingrc` files (see [#58])

## 0.0.1-alpha.1 - 2026-03-19

Initial alpha release

[Break Versioning]: https://www.taoensso.com/break-versioning
[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/

[#3]: https://github.com/aaronmallen/doing/issues/3
[#4]: https://github.com/aaronmallen/doing/issues/4
[#5]: https://github.com/aaronmallen/doing/issues/5
[#6]: https://github.com/aaronmallen/doing/issues/6
[#7]: https://github.com/aaronmallen/doing/issues/7
[#14]: https://github.com/aaronmallen/doing/issues/14
[#17]: https://github.com/aaronmallen/doing/issues/17
[#18]: https://github.com/aaronmallen/doing/issues/18
[#19]: https://github.com/aaronmallen/doing/issues/19
[#20]: https://github.com/aaronmallen/doing/issues/20
[#21]: https://github.com/aaronmallen/doing/issues/21
[#22]: https://github.com/aaronmallen/doing/issues/22
[#23]: https://github.com/aaronmallen/doing/issues/23
[#24]: https://github.com/aaronmallen/doing/issues/24
[#25]: https://github.com/aaronmallen/doing/issues/25
[#26]: https://github.com/aaronmallen/doing/issues/26
[#27]: https://github.com/aaronmallen/doing/issues/27
[#28]: https://github.com/aaronmallen/doing/issues/28
[#30]: https://github.com/aaronmallen/doing/issues/30
[#31]: https://github.com/aaronmallen/doing/issues/31
[#33]: https://github.com/aaronmallen/doing/issues/33
[#36]: https://github.com/aaronmallen/doing/issues/36
[#38]: https://github.com/aaronmallen/doing/issues/38
[#44]: https://github.com/aaronmallen/doing/issues/44
[#46]: https://github.com/aaronmallen/doing/issues/46
[#48]: https://github.com/aaronmallen/doing/issues/48
[#52]: https://github.com/aaronmallen/doing/issues/52
[#58]: https://github.com/aaronmallen/doing/issues/58
[#63]: https://github.com/aaronmallen/doing/issues/63
[#64]: https://github.com/aaronmallen/doing/issues/64
[#66]: https://github.com/aaronmallen/doing/issues/66
[#67]: https://github.com/aaronmallen/doing/issues/67
[#68]: https://github.com/aaronmallen/doing/issues/68
[#70]: https://github.com/aaronmallen/doing/issues/70
[#71]: https://github.com/aaronmallen/doing/issues/71
[#72]: https://github.com/aaronmallen/doing/issues/72
[#79]: https://github.com/aaronmallen/doing/issues/79
[#86]: https://github.com/aaronmallen/doing/issues/86

[Unreleased]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.3...main
[v0.0.1-alpha.2]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.1...0.0.1-alpha.2
[v0.0.1-alpha.3]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.2...0.0.1-alpha.3
