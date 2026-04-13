# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Break Versioning].

## [Unreleased]

## [v0.2.0] - 2026-04-13

### Added

- Natural date expressions like "thirteen days ago" or "twenty minutes ago" now parse correctly, and
  "1 month ago" uses proper calendar subtraction instead of subtracting a fixed 30 days (see [#328])

### Changed

- `note` command can now annotate finished entries; use `--unfinished` to restore the previous
  behavior of only showing unfinished entries (see [#313])
- `last` command now shows the most recent entry regardless of status; use `--unfinished` to restore
  the previous behavior (see [#309])
- `again` command now supports the bare-value filter fallback it was previously missing (see [#367])
- **Breaking (`doing-taskpaper`):** `all_entries` and `entries_in_section` now return iterators
  instead of `Vec<&Entry>` to avoid unnecessary allocations (see [#403])
- **Breaking (`doing-taskpaper`):** Removed `end_date()` method; use `done_date()` directly
  (see [#383])

## [v0.1.13] - 2026-04-04

### Changed

- Listing and searching commands are faster on large doing files thanks to reduced sorting overhead
  and fewer temporary allocations during text matching (see [#408])
- **Breaking (`doing-plugins`):** `Registry::register` and `default_registry` now return `Result`
  instead of panicking on invalid trigger patterns (see [#406])

### Fixed

- `template` command no longer allows path traversal in template names, preventing reads or writes
  outside the configured templates directory (see [#405])
- Undo and redo restores now use unique temporary files instead of a fixed `.tmp` path, preventing
  collisions and stale residue from interrupted operations (see [#407])

## [v0.1.12] - 2026-04-02

### Changed

- Consolidated duplicated filter, display, and entry-lookup logic across CLI commands into shared
  helpers, reducing code duplication by ~330 lines (see [#393], [#394], [#395], [#397])
- Unified test setup boilerplate across command modules into shared test helpers (see [#396])
- Entry grouping in the plugins crate now uses a single generic function instead of separate
  per-key implementations (see [#398])

## [v0.1.11] - 2026-04-01

### Changed

- **Breaking:** `serialize()` in `doing-taskpaper` no longer implicitly deduplicates entries; callers
  that need dedup must call `doc.dedup()` before serializing (see [#399])
- Wildcard pattern matching and document saving are faster due to reduced heap allocations and
  eliminated redundant cloning (see [#399], [#400])

### Fixed

- `finish` command now skips already-done entries, working through unfinished entries in order (see [#404])
- Replaced several `unwrap()` and `unreachable!()` calls with safe fallbacks to prevent panics on
  edge-case inputs (see [#402])

## [v0.1.10] - 2026-04-01

### Added

- `--totals-format` flag on totals-capable commands to switch between `total` and `average` display modes (see [#363])
- `--by` flag on totals-capable commands to group totals by tag or section (see [#365])
- `natural_duration` now includes an "about 20 minutes" bucket for more precise human-readable durations (see [#333])
- Config discovery limits upward directory walk depth to avoid scanning unrelated parent directories (see [#334])

## [v0.1.9] - 2026-03-31

### Changed

- Template parsing and wildcard regex results are now cached, avoiding redundant work per entry (see [#324])
- Reduced heap allocations and redundant computation in search matching, tag display, config loading,
  and JSON export (see [#378], [#380], [#381])

## [v0.1.8] - 2026-03-30

### Added

- Ctrl-C during interactive prompts now cancels cleanly instead of producing an error (see [#323])

### Changed

- **Breaking:** `Note::from_str` renamed to `Note::from_text` in the `doing-taskpaper` crate; `FromStr`
  trait is now implemented for `Note` and `FromIterator<Tag>` for `Tags` (see [#384])
- **Breaking:** `backup_prefix` now returns `Result<String>` instead of `String` in the `doing-ops`
  crate, and `redo` with count 0 now returns an error matching `undo` behavior (see [#386])

### Fixed

- `update` command downloads the binary archive instead of the checksum file (see [#390])
- CSV export correctly escapes values containing embedded quotes (see [#385])
- Several commands no longer panic on edge cases due to unchecked `unwrap()` calls (see [#382])

## [v0.1.7] - 2026-03-30

### Changed

- Unified export and import plugin registries into a single generic registry with a shared `Plugin`
  trait, replacing the separate `ExportRegistry` and `ImportRegistry` types (see [#352], [#353],
  [#354], [#356])

## [v0.1.6] - 2026-03-29

### Changed

- Improved performance of note compression and search/autotag operations (see [#357], [#358])

### Fixed

- `plugins --type` now returns an error for invalid type values instead of silent empty output (see [#351])
- `sections list` respects the `--quiet` flag (see [#349])
- `mark --date` and `tag --date` now produce consistent `YYYY-MM-DD HH:MM` date formats (see [#347])
- `changes` command correctly parses indented continuation lines in changelog entries (see [#344])

## [v0.1.5] - 2026-03-27

### Fixed

- `self-update` no longer fails with a 404 error due to incorrect `v`-prefixed release tag lookup
- `tag-dir --editor` now uses the full editor resolution chain (`DOING_EDITOR`, `VISUAL`,
  `EDITOR`, config) instead of only checking the config default (see [#355])
- `cancel` and `finish` report the actual number of modified entries instead of counting
  silently skipped `never_finish` entries (see [#350])
- `import --from` no longer applies date filter twice when combined with advanced filter flags (see [#346])
- Tag rename operations no longer produce duplicate tags when the target name already exists (see [#343])
- Template `width` setting is now correctly applied when combined with `wrap` on title tokens (see [#341])

## [v0.1.4] - 2026-03-27

### Fixed

- `last --editor` and `select --editor` now persist edits back to the doing file (see [#336], [#337])
- Tag queries no longer panic when string operators are used against Date, Duration, or Time properties (see [#338])
- `budget remove` propagates I/O errors instead of silently ignoring them (see [#339])
- Short date format is no longer incorrectly applied to cross-year dates (see [#340])
- Color validation returns the correct span length after underscore normalization (see [#342])
- Date filters on `on`, `today`, and `yesterday` no longer silently drop entries during DST transitions (see [#345])
- Template parser uses saturating arithmetic to prevent panics on large width or count values (see [#361])

## [v0.1.3] - 2026-03-26

### Changed

- Reduced unnecessary allocations in TaskPaper serializer and note rendering (see [#325])

### Fixed

- Pager is no longer launched when stdout is not a TTY (see [#335])
- Duration parsing rejects extremely large values instead of overflowing (see [#332])
- `again` command propagates real errors instead of masking them as "Skipped" (see [#331])
- ANSI stripping now removes all CSI escape sequences, not just SGR color codes (see [#329])
- Filtered entry lists break sort ties by title for stable display order (see [#327])
- Reversed date ranges are automatically normalized by swapping bounds (see [#322])
- `config set` propagates backup failures instead of silently ignoring them (see [#318])

## [v0.1.2] - 2026-03-25

### Fixed

- "this monday" (and other weekdays) now resolves to the current week instead of 7 days ago (see [#308])
- Single-dash date range separator no longer misparses ISO date hyphens (see [#307])
- Case-insensitive tags now hash correctly for use in HashSet and HashMap collections (see [#310])
- Comments and blank lines between sections are preserved during parse instead of being silently dropped (see [#314])
- ICS calendar import correctly handles TZID timezone parameters and `\;`/`\N` escape sequences (see [#315])
- Control character sentinels replaced with Unicode Private Use Area codepoints to prevent user data
  corruption (see [#316])
- `select` "again" action now preserves tags and notes from the source entry (see [#317])
- Empty editor input returns an error instead of creating a blank entry (see [#319])
- JSON export preserves section order instead of sorting alphabetically (see [#320])
- Unset `HOME` environment variable returns a config error instead of panicking (see [#321])
- Tag search matches each tag individually instead of joining them, preventing cross-tag false positives (see [#326])
- Undo restoration is atomic and propagates directory-read errors (see [#330])
- Install script uses correct checksum filename and strips leading `v` from version

## [v0.1.1] - 2026-03-24

### Fixed

- Entries with DST-ambiguous timestamps (spring-forward gaps and fall-back folds) are preserved instead of
  silently dropped by @havenwood (see [#299], [#300])
- `beginning_of_day` no longer panics when midnight and 1 AM both fall in a DST gap (see [#302])
- Duplicate section headers in a doing file are merged instead of silently discarding entries (see [#301])
- `grep --editor` now persists edits back to the doing file (see [#303])
- Empty or whitespace-only `EDITOR` variable returns a clear error instead of panicking (see [#304])
- ICS calendar import preserves long SUMMARY and DESCRIPTION values with RFC 5545 continuation lines (see [#305])
- CJK characters and emoji no longer misalign columns in template output (see [#306])
- Backup file discovery is stable across Rust toolchain upgrades (see [#311])

## [v0.1.0] - 2026-03-23

### Added

- `completion` command for shell completion generation (see [#273])
- `man` subcommand generates man pages (see [#274])
- "Did you mean?" suggestions for unknown subcommands (see [#276])
- `--exact`/`-x` flag on `finish` and `cancel` for exact string matching (see [#279])
- Missing short flags to match Ruby CLI: `archive -t`, `done -t`, `redo -f`, `select -q`, filter `-a` (see [#281])
- `undo` gains `--file`, `--prune`, and `--redo` flags; `redo` gains count argument and `--interactive`
  (see [#174], [#175], [#176], [#177])
- `select` gains filter flags (`--case`, `--exact`, `--not`, `--val`), date filters
  (`--after`/`--before`/`--from`), and action flags (`--again`/`--remove`/`--force`)
  (see [#178], [#179], [#180])
- `config dump`/`open` aliases, `--output json/yaml` on `config get`, `config undo` subcommand, and
  fuzzy key path matching (see [#181], [#182], [#183])
- `rotate` applies full filter pipeline and per-section `--keep` (see [#184])
- `import` gains `--after`, `--before`, `--case`, `--exact`, `--not`, `--only-timed` filter flags (see [#186])
- `tag-dir` gains `--remove`, `--clear`, `--editor` flags (see [#187])
- `commands` gains `ls` alias, `--disabled`, `--style` flags (see [#190])
- `plugins` gains `--type` and `--column` flags (see [#191])
- `note` supports reading note text from stdin (see [#193])
- `show --menu`/`-m` flag for interactive section selection (see [#157])

### Changed

- Codebase restructured into Cargo workspace with independent crates (see [#214])
- Improved performance for tag lookups, title parsing, dedup, and group-by operations (see [#292])

### Fixed

- Graceful error handling replaces panics on invalid editor paths, missing HOME, and completions (see [#293])
- UTF-8 multibyte characters no longer cause truncation panics in template rendering (see [#294])
- `views remove` changed from subcommand to `--remove`/`-r` flag to match Ruby interface (see [#280])
- `done` targets the most recent entry regardless of `@done` status, matching Ruby behavior (see [#278])
- `-X` (uppercase) restored for `--noauto`, `-x` (lowercase) consistently available for `--exact` (see [#277])

## [v0.0.1-alpha.7] - 2026-03-23

### Added

- `--auto` flag on `finish` to auto-generate `@done` timestamps based on entry sequence (see [#168])
- `--from` flag on `now`, `done`, and `finish` supports date range parsing (see [#160])
- `--template` accepts inline template strings; `--config-template` selects named templates from config (see [#158])
- Ruby-style `key = value` mapping syntax in autotag configuration (see [#163])
- `--autotag` flag on `tag` to apply autotag rules from config (see [#205])
- `show` accepts `@tag` as first positional argument for tag filtering (see [#202])
- `--case`, `--exact` filter flags and `--bool PATTERN` support on `again` (see [#194], [#195])
- `--case` flag on `finish` for search case sensitivity (see [#171])
- `--stdout` global flag to redirect status messages to stdout (see [#155])
- Config file path included in `--debug` output (see [#153])
- `MM/DD` short date and US date + time combination parsing (see [#151], [#152])
- `--no-times` flag on `today`, `yesterday`, `recent`, `since` (see [#204])
- `changes` alias for changelog command (see [#200])
- Missing `-c`, `-n`, `-t` short flags on `mark` and `reset` (see [#199])

### Fixed

- `extract_note` matches Ruby doing behavior for multiple parenthetical notes (see [#161])
- `tags` supports `--no-counts`, `--no-line` flags and silences empty output (see [#208], [#209], [#211])

## [v0.0.1-alpha.6.1] - 2026-03-23

### Added

- `--hilite` flag on display commands for search match highlighting (see [#206])
- `--column`/`-c` and `--output`/`-o` flags on `views`, positional name argument to dump view config (see [#189])
- `--to` flag on `archive` to move entries to a specified section instead of Archive (see [#188])
- Status message on `now` matches Ruby doing format (see [#159])

### Fixed

- `--output json` on `view` uses export plugin pipeline instead of template rendering (see [#206])
- `show_template` falls through to built-in handling when template file is empty; `--no-column` flag disables column
  format (see [#203])
- `on` date range `to` boundary is now inclusive for date-only expressions (see [#201])
- `note --remove` with text replaces the note instead of clearing it (see [#198])
- `tags --val` flag now functional (see [#210])
- `archive --keep` ignored when tag or search filters are active (see [#188])
- `cancel --case` flag controls search case sensitivity (see [#197])
- `again` exits 0 with a message when no entries match instead of erroring (see [#196])
- `finish` status message matches Ruby format; `--archive` adds `@from(Section)` tag (see [#164], [#169], [#172])
- `done` outputs "no items matched your search" when no entries to tag (see [#162])
- `--noauto` short flag changed from `-X` to `-x` to match Ruby doing (see [#156])
- `--notes`/`--no-notes` flags control note display in output (see [#154])

## [v0.0.1-alpha.5] - 2026-03-22

### Fixed

- `--back` and `--took` flags on `finish` now correctly adjust entry start time (see [#167], [#173])
- `finish` supports `--bool AND` with comma-separated `--tag` values (see [#170])
- `--val` flag included in filter checks across all commands; bare values fall back to equality checks against the first
  tag (see [#185])
- `finish` count argument selects the N most recent entries by date (see [#166])
- `finish` overwrites existing `@done` dates by default, matching Ruby doing behavior (see [#165])
- `tags` outputs bare tag names instead of `@`-prefixed names in default and `--counts` modes (see [#207])

## [v0.0.1-alpha.4.1] - 2026-03-21

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
[#151]: https://github.com/aaronmallen/doing/issues/151
[#152]: https://github.com/aaronmallen/doing/issues/152
[#153]: https://github.com/aaronmallen/doing/issues/153
[#154]: https://github.com/aaronmallen/doing/issues/154
[#155]: https://github.com/aaronmallen/doing/issues/155
[#156]: https://github.com/aaronmallen/doing/issues/156
[#157]: https://github.com/aaronmallen/doing/issues/157
[#158]: https://github.com/aaronmallen/doing/issues/158
[#159]: https://github.com/aaronmallen/doing/issues/159
[#160]: https://github.com/aaronmallen/doing/issues/160
[#161]: https://github.com/aaronmallen/doing/issues/161
[#162]: https://github.com/aaronmallen/doing/issues/162
[#163]: https://github.com/aaronmallen/doing/issues/163
[#164]: https://github.com/aaronmallen/doing/issues/164
[#165]: https://github.com/aaronmallen/doing/issues/165
[#166]: https://github.com/aaronmallen/doing/issues/166
[#167]: https://github.com/aaronmallen/doing/issues/167
[#168]: https://github.com/aaronmallen/doing/issues/168
[#169]: https://github.com/aaronmallen/doing/issues/169
[#170]: https://github.com/aaronmallen/doing/issues/170
[#171]: https://github.com/aaronmallen/doing/issues/171
[#172]: https://github.com/aaronmallen/doing/issues/172
[#173]: https://github.com/aaronmallen/doing/issues/173
[#174]: https://github.com/aaronmallen/doing/issues/174
[#175]: https://github.com/aaronmallen/doing/issues/175
[#176]: https://github.com/aaronmallen/doing/issues/176
[#177]: https://github.com/aaronmallen/doing/issues/177
[#178]: https://github.com/aaronmallen/doing/issues/178
[#179]: https://github.com/aaronmallen/doing/issues/179
[#180]: https://github.com/aaronmallen/doing/issues/180
[#181]: https://github.com/aaronmallen/doing/issues/181
[#182]: https://github.com/aaronmallen/doing/issues/182
[#183]: https://github.com/aaronmallen/doing/issues/183
[#184]: https://github.com/aaronmallen/doing/issues/184
[#185]: https://github.com/aaronmallen/doing/issues/185
[#186]: https://github.com/aaronmallen/doing/issues/186
[#187]: https://github.com/aaronmallen/doing/issues/187
[#188]: https://github.com/aaronmallen/doing/issues/188
[#189]: https://github.com/aaronmallen/doing/issues/189
[#190]: https://github.com/aaronmallen/doing/issues/190
[#191]: https://github.com/aaronmallen/doing/issues/191
[#193]: https://github.com/aaronmallen/doing/issues/193
[#194]: https://github.com/aaronmallen/doing/issues/194
[#195]: https://github.com/aaronmallen/doing/issues/195
[#196]: https://github.com/aaronmallen/doing/issues/196
[#197]: https://github.com/aaronmallen/doing/issues/197
[#198]: https://github.com/aaronmallen/doing/issues/198
[#199]: https://github.com/aaronmallen/doing/issues/199
[#200]: https://github.com/aaronmallen/doing/issues/200
[#201]: https://github.com/aaronmallen/doing/issues/201
[#202]: https://github.com/aaronmallen/doing/issues/202
[#203]: https://github.com/aaronmallen/doing/issues/203
[#204]: https://github.com/aaronmallen/doing/issues/204
[#205]: https://github.com/aaronmallen/doing/issues/205
[#206]: https://github.com/aaronmallen/doing/issues/206
[#207]: https://github.com/aaronmallen/doing/issues/207
[#208]: https://github.com/aaronmallen/doing/issues/208
[#209]: https://github.com/aaronmallen/doing/issues/209
[#210]: https://github.com/aaronmallen/doing/issues/210
[#211]: https://github.com/aaronmallen/doing/issues/211
[#214]: https://github.com/aaronmallen/doing/issues/214
[#273]: https://github.com/aaronmallen/doing/issues/273
[#274]: https://github.com/aaronmallen/doing/issues/274
[#276]: https://github.com/aaronmallen/doing/issues/276
[#277]: https://github.com/aaronmallen/doing/issues/277
[#278]: https://github.com/aaronmallen/doing/issues/278
[#279]: https://github.com/aaronmallen/doing/issues/279
[#280]: https://github.com/aaronmallen/doing/issues/280
[#281]: https://github.com/aaronmallen/doing/issues/281
[#292]: https://github.com/aaronmallen/doing/issues/292
[#293]: https://github.com/aaronmallen/doing/issues/293
[#294]: https://github.com/aaronmallen/doing/issues/294
[#299]: https://github.com/aaronmallen/doing/issues/299
[#300]: https://github.com/aaronmallen/doing/issues/300
[#301]: https://github.com/aaronmallen/doing/issues/301
[#302]: https://github.com/aaronmallen/doing/issues/302
[#303]: https://github.com/aaronmallen/doing/issues/303
[#304]: https://github.com/aaronmallen/doing/issues/304
[#305]: https://github.com/aaronmallen/doing/issues/305
[#306]: https://github.com/aaronmallen/doing/issues/306
[#307]: https://github.com/aaronmallen/doing/issues/307
[#308]: https://github.com/aaronmallen/doing/issues/308
[#310]: https://github.com/aaronmallen/doing/issues/310
[#311]: https://github.com/aaronmallen/doing/issues/311
[#314]: https://github.com/aaronmallen/doing/issues/314
[#315]: https://github.com/aaronmallen/doing/issues/315
[#316]: https://github.com/aaronmallen/doing/issues/316
[#317]: https://github.com/aaronmallen/doing/issues/317
[#318]: https://github.com/aaronmallen/doing/issues/318
[#319]: https://github.com/aaronmallen/doing/issues/319
[#320]: https://github.com/aaronmallen/doing/issues/320
[#321]: https://github.com/aaronmallen/doing/issues/321
[#322]: https://github.com/aaronmallen/doing/issues/322
[#324]: https://github.com/aaronmallen/doing/issues/324
[#325]: https://github.com/aaronmallen/doing/issues/325
[#326]: https://github.com/aaronmallen/doing/issues/326
[#327]: https://github.com/aaronmallen/doing/issues/327
[#329]: https://github.com/aaronmallen/doing/issues/329
[#330]: https://github.com/aaronmallen/doing/issues/330
[#331]: https://github.com/aaronmallen/doing/issues/331
[#332]: https://github.com/aaronmallen/doing/issues/332
[#335]: https://github.com/aaronmallen/doing/issues/335
[#336]: https://github.com/aaronmallen/doing/issues/336
[#337]: https://github.com/aaronmallen/doing/issues/337
[#338]: https://github.com/aaronmallen/doing/issues/338
[#339]: https://github.com/aaronmallen/doing/issues/339
[#340]: https://github.com/aaronmallen/doing/issues/340
[#341]: https://github.com/aaronmallen/doing/issues/341
[#342]: https://github.com/aaronmallen/doing/issues/342
[#343]: https://github.com/aaronmallen/doing/issues/343
[#344]: https://github.com/aaronmallen/doing/issues/344
[#345]: https://github.com/aaronmallen/doing/issues/345
[#346]: https://github.com/aaronmallen/doing/issues/346
[#347]: https://github.com/aaronmallen/doing/issues/347
[#349]: https://github.com/aaronmallen/doing/issues/349
[#350]: https://github.com/aaronmallen/doing/issues/350
[#351]: https://github.com/aaronmallen/doing/issues/351
[#352]: https://github.com/aaronmallen/doing/issues/352
[#353]: https://github.com/aaronmallen/doing/issues/353
[#354]: https://github.com/aaronmallen/doing/issues/354
[#355]: https://github.com/aaronmallen/doing/issues/355
[#356]: https://github.com/aaronmallen/doing/issues/356
[#357]: https://github.com/aaronmallen/doing/issues/357
[#358]: https://github.com/aaronmallen/doing/issues/358
[#323]: https://github.com/aaronmallen/doing/issues/323
[#361]: https://github.com/aaronmallen/doing/issues/361
[#382]: https://github.com/aaronmallen/doing/issues/382
[#384]: https://github.com/aaronmallen/doing/issues/384
[#385]: https://github.com/aaronmallen/doing/issues/385
[#378]: https://github.com/aaronmallen/doing/issues/378
[#380]: https://github.com/aaronmallen/doing/issues/380
[#381]: https://github.com/aaronmallen/doing/issues/381
[#386]: https://github.com/aaronmallen/doing/issues/386
[#309]: https://github.com/aaronmallen/doing/issues/309
[#313]: https://github.com/aaronmallen/doing/issues/313
[#328]: https://github.com/aaronmallen/doing/issues/328
[#333]: https://github.com/aaronmallen/doing/issues/333
[#334]: https://github.com/aaronmallen/doing/issues/334
[#363]: https://github.com/aaronmallen/doing/issues/363
[#365]: https://github.com/aaronmallen/doing/issues/365
[#367]: https://github.com/aaronmallen/doing/issues/367
[#383]: https://github.com/aaronmallen/doing/issues/383
[#390]: https://github.com/aaronmallen/doing/issues/390
[#399]: https://github.com/aaronmallen/doing/issues/399
[#400]: https://github.com/aaronmallen/doing/issues/400
[#402]: https://github.com/aaronmallen/doing/issues/402
[#404]: https://github.com/aaronmallen/doing/issues/404
[#405]: https://github.com/aaronmallen/doing/issues/405
[#406]: https://github.com/aaronmallen/doing/issues/406
[#407]: https://github.com/aaronmallen/doing/issues/407
[#408]: https://github.com/aaronmallen/doing/issues/408
[#393]: https://github.com/aaronmallen/doing/issues/393
[#394]: https://github.com/aaronmallen/doing/issues/394
[#395]: https://github.com/aaronmallen/doing/issues/395
[#396]: https://github.com/aaronmallen/doing/issues/396
[#397]: https://github.com/aaronmallen/doing/issues/397
[#398]: https://github.com/aaronmallen/doing/issues/398
[#403]: https://github.com/aaronmallen/doing/issues/403

[Unreleased]: https://github.com/aaronmallen/doing/compare/0.2.0...main
[v0.0.1-alpha.2]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.1...0.0.1-alpha.2
[v0.0.1-alpha.3]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.2...0.0.1-alpha.3
[v0.0.1-alpha.4.1]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.3...0.0.1-alpha.4.1
[v0.0.1-alpha.5]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.4.1...0.0.1-alpha.5
[v0.0.1-alpha.6.1]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.5...0.0.1-alpha.6.1
[v0.0.1-alpha.7]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.6.1...0.0.1-alpha.7
[v0.1.0]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.7...0.1.0
[v0.1.1]: https://github.com/aaronmallen/doing/compare/0.1.0...0.1.1
[v0.1.2]: https://github.com/aaronmallen/doing/compare/0.1.1...0.1.2
[v0.1.3]: https://github.com/aaronmallen/doing/compare/0.1.2...0.1.3
[v0.1.4]: https://github.com/aaronmallen/doing/compare/0.1.3...0.1.4
[v0.1.5]: https://github.com/aaronmallen/doing/compare/0.1.4...0.1.5
[v0.1.6]: https://github.com/aaronmallen/doing/compare/0.1.5...0.1.6
[v0.1.7]: https://github.com/aaronmallen/doing/compare/0.1.6...0.1.7
[v0.1.8]: https://github.com/aaronmallen/doing/compare/0.1.7...0.1.8
[v0.1.9]: https://github.com/aaronmallen/doing/compare/0.1.8...0.1.9
[v0.1.10]: https://github.com/aaronmallen/doing/compare/0.1.9...0.1.10
[v0.1.11]: https://github.com/aaronmallen/doing/compare/0.1.10...0.1.11
[v0.1.12]: https://github.com/aaronmallen/doing/compare/0.1.11...0.1.12
[v0.1.13]: https://github.com/aaronmallen/doing/compare/0.1.12...0.1.13
[v0.2.0]: https://github.com/aaronmallen/doing/compare/0.1.13...0.2.0
