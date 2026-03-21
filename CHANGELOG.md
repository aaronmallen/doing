# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Break Versioning].

## [Unreleased]

## [v0.0.1-alpha.4] - 2026-03-21

### Added

- `--title` flag on display commands accepts an optional string value to set a custom section header (see [#32], [#88])
- Day One export formats: `dayone`, `dayone-days`, and `dayone-entries` for Day One importable JSON (see [#55])
- Calendar (ICS) and JSON import formats (see [#54])
- `--output timeline` format renders entries as a self-contained HTML timeline visualization (see [#43])
- Prompt before creating new sections in interactive mode (see [#40])
- Parenthetical notes in entry titles are automatically extracted as notes (see [#12])
- `--from` flag accepts a single date in addition to date ranges (see [#9])
- Fuzzy prefix matching for view names (see [#15])
- `--not` and `--val` flags on `finish`, `cancel`, and `again` commands (see [#80])
- Positional count argument for `finish`, `cancel`, and `recent` (see [#34])
- `--exact` flag on all filter commands for exact string matching (see [#29])
- `--tag` flag accepts comma-separated values (see [#11])
- `--case` flag on all filter commands for case sensitivity override (see [#10])
- `--tag_sort` and `--tag_order` flags on `show` (see [#8])
- `config set --local` flag to write to the local `.doingrc` (see [#57])
- `config edit` gains `--app`, `--editor`, `--bundle_id`, and `--default` flags (see [#56])
- `config set --remove` flag to delete a config key (see [#16])
- `update` command for self-updating the binary (see [#62])
- `changes` command to display formatted changelog history (see [#61])
- Visual color swatches in `colors` command output (see [#49])
- `sections remove --archive` flag to archive entries before removing a section (see [#83])
- `open` gains `--editor` and `--bundle_id` flags (see [#53], [#81])
- `last` gains `--delete` and `--editor` flags (see [#77])
- `grep` gains `--delete` and `--editor` flags (see [#76])
- `finish --back` flag for backdating completion time (see [#2], [#75])
- `meanwhile --archive` flag to archive finished entries (see [#37])

### Fixed

- Short flag conflicts resolved to align with Ruby doing conventions (see [#100])
- `--count` scoped per-command instead of shared, restoring `-c` short flag on individual commands
- `extract_note` no longer strips tag values like `@project(myapp)`
- Markdown export removes top-level heading and uses abbreviated date format; TaskPaper renders flat entry list
  (see [#73])
- HTML export wraps `@done(date)` tags in a single `<span>` element (see [#59])
- CSV output format matches Ruby doing (fixed date format, raw timer seconds, quoted fields) (see [#45])
- `--ask` prompts for note on `now`, `done`, `meanwhile`, `again`, and `note` commands (see [#78])
- `--only-timed` excludes zero-duration entries (see [#42])
- `tag --date` includes time in the value (see [#39])
- `grep --duration` changed from format string to boolean toggle (see [#69])
- `commands` command behavior matches Ruby doing (see [#51])
- `template` command lists export templates by default (see [#50])
- `today` respects the `--totals` flag (see [#41])
- `reset` gains positional date argument and `--took` flag (see [#82])
- `tags` adds `@` prefix to output, sorts by count, and gains max count argument and filter flags (see [#74])
- `archive` accepts positional `[SECTION_OR_TAG]` argument (see [#65])

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

[#2]: https://github.com/aaronmallen/doing/issues/2
[#3]: https://github.com/aaronmallen/doing/issues/3
[#4]: https://github.com/aaronmallen/doing/issues/4
[#5]: https://github.com/aaronmallen/doing/issues/5
[#6]: https://github.com/aaronmallen/doing/issues/6
[#7]: https://github.com/aaronmallen/doing/issues/7
[#8]: https://github.com/aaronmallen/doing/issues/8
[#9]: https://github.com/aaronmallen/doing/issues/9
[#10]: https://github.com/aaronmallen/doing/issues/10
[#11]: https://github.com/aaronmallen/doing/issues/11
[#12]: https://github.com/aaronmallen/doing/issues/12
[#14]: https://github.com/aaronmallen/doing/issues/14
[#15]: https://github.com/aaronmallen/doing/issues/15
[#16]: https://github.com/aaronmallen/doing/issues/16
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
[#29]: https://github.com/aaronmallen/doing/issues/29
[#30]: https://github.com/aaronmallen/doing/issues/30
[#31]: https://github.com/aaronmallen/doing/issues/31
[#32]: https://github.com/aaronmallen/doing/issues/32
[#33]: https://github.com/aaronmallen/doing/issues/33
[#34]: https://github.com/aaronmallen/doing/issues/34
[#36]: https://github.com/aaronmallen/doing/issues/36
[#37]: https://github.com/aaronmallen/doing/issues/37
[#38]: https://github.com/aaronmallen/doing/issues/38
[#39]: https://github.com/aaronmallen/doing/issues/39
[#40]: https://github.com/aaronmallen/doing/issues/40
[#41]: https://github.com/aaronmallen/doing/issues/41
[#42]: https://github.com/aaronmallen/doing/issues/42
[#43]: https://github.com/aaronmallen/doing/issues/43
[#44]: https://github.com/aaronmallen/doing/issues/44
[#45]: https://github.com/aaronmallen/doing/issues/45
[#46]: https://github.com/aaronmallen/doing/issues/46
[#48]: https://github.com/aaronmallen/doing/issues/48
[#49]: https://github.com/aaronmallen/doing/issues/49
[#50]: https://github.com/aaronmallen/doing/issues/50
[#51]: https://github.com/aaronmallen/doing/issues/51
[#52]: https://github.com/aaronmallen/doing/issues/52
[#53]: https://github.com/aaronmallen/doing/issues/53
[#54]: https://github.com/aaronmallen/doing/issues/54
[#55]: https://github.com/aaronmallen/doing/issues/55
[#56]: https://github.com/aaronmallen/doing/issues/56
[#57]: https://github.com/aaronmallen/doing/issues/57
[#58]: https://github.com/aaronmallen/doing/issues/58
[#59]: https://github.com/aaronmallen/doing/issues/59
[#61]: https://github.com/aaronmallen/doing/issues/61
[#62]: https://github.com/aaronmallen/doing/issues/62
[#63]: https://github.com/aaronmallen/doing/issues/63
[#64]: https://github.com/aaronmallen/doing/issues/64
[#65]: https://github.com/aaronmallen/doing/issues/65
[#66]: https://github.com/aaronmallen/doing/issues/66
[#67]: https://github.com/aaronmallen/doing/issues/67
[#68]: https://github.com/aaronmallen/doing/issues/68
[#69]: https://github.com/aaronmallen/doing/issues/69
[#70]: https://github.com/aaronmallen/doing/issues/70
[#71]: https://github.com/aaronmallen/doing/issues/71
[#72]: https://github.com/aaronmallen/doing/issues/72
[#73]: https://github.com/aaronmallen/doing/issues/73
[#74]: https://github.com/aaronmallen/doing/issues/74
[#75]: https://github.com/aaronmallen/doing/issues/75
[#76]: https://github.com/aaronmallen/doing/issues/76
[#77]: https://github.com/aaronmallen/doing/issues/77
[#78]: https://github.com/aaronmallen/doing/issues/78
[#79]: https://github.com/aaronmallen/doing/issues/79
[#80]: https://github.com/aaronmallen/doing/issues/80
[#81]: https://github.com/aaronmallen/doing/issues/81
[#82]: https://github.com/aaronmallen/doing/issues/82
[#83]: https://github.com/aaronmallen/doing/issues/83
[#86]: https://github.com/aaronmallen/doing/issues/86
[#88]: https://github.com/aaronmallen/doing/issues/88
[#100]: https://github.com/aaronmallen/doing/issues/100

[Unreleased]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.4...main
[v0.0.1-alpha.2]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.1...0.0.1-alpha.2
[v0.0.1-alpha.3]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.2...0.0.1-alpha.3
[v0.0.1-alpha.4]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.3...0.0.1-alpha.4
