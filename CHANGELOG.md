# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Break Versioning].

## [Unreleased]

### Added

- [#2] `-b`/`--back` flag on `finish` command to backdate `@done` timestamp using natural language
  (e.g. `--back "30m ago"`, `--back "2024-03-17 14:00"`); `--back` and `--at` are mutually exclusive
- [#8] `--tag_sort` (name/time) and `--tag_order` (asc/desc) flags on `show` command for controlling tag totals
  sort order
- [#9] `--from` flag accepts a single date (e.g. `--from monday`, `--from "march 15"`) and returns a 24-hour span
  instead of requiring a range separator
- [#12] Parenthetical notes in entry titles are automatically extracted as notes
  (e.g. `doing now "Working on feature (some context)"` attaches "some context" as a note)
- [#10] `--case` flag on filter commands (show, grep, etc.) to override search case sensitivity; moved from
  grep-specific to shared `FilterArgs`
- [#11] `--tag` flag accepts comma-separated values (e.g. `--tag project,urgent`) in addition to repeated flags
- [#15] Fuzzy prefix matching for view names; `doing view co` resolves to `doing view coding` when unambiguous,
  and returns an error listing candidates when ambiguous
- [#16] `-r`/`--remove` flag on `config set` to remove configuration keys (e.g. `doing config set -r plugins.say`)
- [#29] `-x`/`--exact` flag on filter commands for exact string matching; moved from grep-specific to shared
  `FilterArgs`
- [#32] `--title` flag on display commands to show section headers in output
- [#34] Positional count argument on `finish`, `cancel`, and `recent` commands (e.g. `doing finish 3`); takes
  precedence over `--count` when the flag is not explicitly set
- [#37] `--archive` flag on `meanwhile` command; moves finished @meanwhile entries to the Archive section when replacing
- [#40] Interactive prompt before creating new sections; respects `--yes`/`--no`/`--default` flags on `now`, `done`,
  `meanwhile`, and `again` commands
- [#43] `--output timeline` format generates a self-contained HTML page with a visual timeline showing entry durations
  with proportional bars and start/end times
- [#54] Calendar (ICS) and JSON import formats for the `import` command; `--type calendar` imports VEVENT entries,
  `--type json` re-imports doing's own JSON export format
- [#55] `dayone`, `dayone-days`, and `dayone-entries` export plugins for Day One importable JSON output
- [#56] `-a`/`--app`, `-e`/`--editor`, `-b`/`--bundle_id`, and `-x`/`--default` flags on `config edit`; open config
  with a specific app/editor or reset to defaults
- [#57] `-l`/`--local` flag on `config set` to write values to `.doingrc` in the current directory instead of the
  global config
- [#61] `changes` command displays a formatted changelog of recent doing versions with `--all`, `--lookup`,
  `--search`, `--only`, `--sort`, `--markdown`, `--changes`, `--prefix`, and `--interactive` flags; `changelog` is a
  visible alias
- [#62] `update` command checks for the latest GitHub release and self-updates the binary after confirmation
- [#76] `-d`/`--delete` and `-e`/`--editor` flags on `grep` command; `--delete` removes all matching entries,
  `--editor` opens matches in an editor for batch editing; both respect `--interactive` for selective action
- [#77] `-d`/`--delete` and `-e`/`--editor` flags on `last` command; `--delete` removes the last entry from the doing
  file and `--editor` opens it in `$EDITOR` for modification
- [#80] `--not` and `--val` flags on `finish`, `cancel`, and `again` commands for tag negation and value filtering
- [#81] `-e`/`--editor` flag on `open` command to override the default editor
- [#83] `sections remove --archive` flag to archive entries before removing a section

### Changed

- [#49] `colors` command displays visual color swatches with labels, organized by category (foreground, bold/bright,
  background, modifiers, themes, reset/default) instead of a flat list

### Fixed

- [#39] `tag --date` only includes date in tag value; now includes time (`%Y-%m-%d %H:%M`) to match Ruby doing
- [#41] `today --totals` does not display tag time totals; totals now render consistently with `show --totals`
- [#42] `--only-timed` includes entries with zero-duration intervals; now correctly excludes entries where
  start and end times are identical
- [#45] CSV output format deviations: dates now include seconds and timezone, timer is raw seconds, all fields quoted
- [#47] `today --output json` already uses full ISO dates; added integration test to lock in correct behavior
- [#50] `template` command managed display templates instead of export format templates; now lists, displays, and saves
  export templates (HTML CSS, etc.) matching Ruby doing behavior with `--list`, `--path`, `--save`, and `--column` flags
- [#51] `commands` command lists all subcommands (help-overview) instead of managing optional commands; now supports
  `list`, `add`/`enable`, `remove`/`disable` subcommands to control which commands are available via `disabled_commands`
  config
- [#53] `-b` short flag on `open` command reassigned from `--backup` to `--bundle_id` to match Ruby doing;
  `--backup` no longer has a short flag; `--bundle_id` accepts a macOS bundle identifier string
- [#59] HTML output breaks `<span>` tags for `@done(date)` entries; regex now correctly wraps the tag name without
  consuming the parenthetical value
- [#65] `archive` command missing positional `[SECTION_OR_TAG]` argument; `doing archive Currently` and
  `doing archive @done` now work without requiring `--section`/`--tag` flags
- [#69] `grep --duration` accepted a format string argument instead of acting as a boolean toggle; now matches Ruby
  doing's `--[no-]duration` behavior
- [#73] Markdown output included unnecessary top-level heading and wrong date format; TaskPaper output included
  section headers and tab indentation not matching Ruby doing
- [#74] `tags` command output missing `@` prefix on tag names, `--sort count` not supported, no positional
  `MAX_COUNT` argument, and missing filtering flags (`--search`, `--tag`, `--bool`, `--val`, `--not`)
- [#78] `--ask` flag on `now` incorrectly prompted for title instead of note; added `--ask` to `done`, `meanwhile`,
  `again`, and `note` commands
- [#82] `reset` command missing positional `[DATE_STRING]` argument and `--took`/`--for` flag; `doing reset 3pm` and
  `doing reset "1 hour ago" --took 1h30m` now work as expected

## [v0.0.1-alpha.3] - 2026-03-20

### Added

- [#14] Shorthand duration expressions (`24h`, `48h`, `30m`, `2h`, `1d2h`) are now accepted anywhere a natural language
  time expression is used (e.g. `--back 24h`)
- [#27] `DOING_FILE` environment variable to specify the doing file path; `-f`/`--doing-file` takes precedence when both
  are set
- [#28] `--no-pager` global flag to explicitly disable pager output when `paginate` is enabled in config; `--pager` and
  `--no-pager` now override config defaults
- [#30] `-i`/`--interactive` flag on `again`, `cancel`, `finish`, `grep`, `mark`, `note`, `recent`, `reset`, `show`, and
  `tag` commands; presents a selection menu (fzf if available, built-in fallback) for choosing entries to act on
- [#31] `-u`/`--unfinished` flag on `cancel`, `finish`, `mark`, and `tag` commands; when passed, only entries without a
  `@done` tag are included as candidates

### Fixed

- [#3] `show --times` flag was not wired into rendering; finished entries now display time intervals
- [#4] `show --section` flag was ignored; section selection now respects `--section` flag when positional arg is omitted
- [#5] `rotate` archive filename uses monthly (`YYYY-MM`) suffix instead of daily (`YYYY-MM-DD`) to match Ruby doing
- [#6] Sequential `doing undo` calls restore the same backup instead of walking backwards through history; undo now
  consumes backups (`.bak` → `.undone`) and redo restores the most recent consumed backup
- [#7] `reset --from "8am to 10am"` filters by date range instead of setting start/done times to match Ruby doing
- [#17] `--output` with an unrecognized format silently falls back to template rendering instead of reporting an error
- [#22] `grep` only searches entry titles and notes, missing entries that match by tag name (e.g. `doing grep "code"`
  does not find entries tagged `@code`)
- [#25] `--bool` flag requires exact lowercase values; `--bool AND` and `--bool Or` are rejected
- [#36] `archive` command does not add `@from(SectionName)` tag by default; `--label` was opt-in instead of default
- [#38] `meanwhile` does not remove `@meanwhile` tag from finished entry when a new meanwhile replaces it
- [#46] Default display template has multiple formatting deviations from Ruby doing: adds `║` separator, left-aligned
  section labels in brackets (`[Currently ]`), `HH:MM:SS` clock duration format, interval shown by default on `@done`
  entries, `last` and `yesterday` use simplified template without section/interval, and `--times` flag removed
- [#48] Status messages use structured log format (`[timestamp INFO module::path]`) instead of user-friendly output;
  status messages are now printed directly to stderr as clean text, independent of the logging system
- [#52] `budget` argument order reversed from Ruby doing; now accepts `doing budget TAG [AMOUNT]` with tag first;
  `doing budget TAG` without an amount shows current usage for that tag
- [#63] Display output missing trailing newline; shell `%` / `⏎` indicator appears after display commands
- [#64] `again` with no filters only considers unfinished entries; now selects the most recent entry regardless of
  `@done` status so previously completed tasks can be resumed
- [#66] `--at` flag resolves bare times ("2pm", "3:30pm") to yesterday instead of today
- [#67] `--bool` flag defaults to `or` instead of `pattern`; `+`/`-` tag prefixes now work correctly without explicit
  `--bool pattern`
- [#68] `done` with no arguments fails to find last unfinished entry when last entry is already `@done`
- [#70] `--noauto` short flag is lowercase `-x` instead of uppercase `-X`; conflicts with `-x` for `--exact`
- [#71] `on` command fails to parse bare day-of-week names; `doing on friday` and `doing on fri` now correctly resolve
  to the most recent past occurrence of that weekday
- [#72] `-t` short flag maps to `--tag` instead of `--times` on display commands; `-t` now maps to `--times` and
  `--tag` has no short flag on `show`, `grep`, `today`, `yesterday`, `recent`, `since`, and `on`
- [#79] `--back` missing `--started`/`--since` aliases, `--at` missing `--finished` alias, and `--took` missing
  `--for` alias; all aliases now available on commands where these flags exist

## [v0.0.1-alpha.2] - 2026-03-20

### Fixed

- Default template was empty, causing `show` and other display commands to produce no output without explicit template
  config
- Default template now includes colors (`%boldwhite`, `%boldcyan`, `%cyan`) matching the Ruby doing gem style
- [#18] `archive` and `rotate` commands only process `@done` entries instead of all entries
- [#19] `again` command selects last `@done` entry instead of last unfinished entry
- [#20] `tag`, `note`, `mark`, and `last` commands operate on `@done` entries instead of skipping them
- [#21] `reset` command does not remove `@done` tag by default, requiring explicit `--resume` flag
- [#23] Undo history is not isolated per doing file; undoing in one file can restore content from a different file
- [#24] `reset` command crashes with `--search`, `--back`, or `--from` flags when search config is not explicitly set
- [#26] Tags are stripped from entry titles in display output and info messages
- [#33] `-s` short flag maps to `--section` instead of `--search` to match Brett's doing; `--search` no longer has a
  short flag
- [#44] JSON output schema does not match Brett's doing format; missing `id`, `done`, `end_date`, `timers` fields,
  wrong top-level structure, date format lacks timezone/seconds, tags serialized as objects instead of strings, and
  title has tags stripped
- [#58] `tag-dir` fails on empty `.doingrc` files; empty config files now treated as empty objects

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
[#47]: https://github.com/aaronmallen/doing/issues/47
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
[#76]: https://github.com/aaronmallen/doing/issues/76
[#77]: https://github.com/aaronmallen/doing/issues/77
[#78]: https://github.com/aaronmallen/doing/issues/78
[#79]: https://github.com/aaronmallen/doing/issues/79
[#80]: https://github.com/aaronmallen/doing/issues/80
[#81]: https://github.com/aaronmallen/doing/issues/81
[#82]: https://github.com/aaronmallen/doing/issues/82
[#83]: https://github.com/aaronmallen/doing/issues/83

[Unreleased]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.3...main
[v0.0.1-alpha.2]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.1...0.0.1-alpha.2
[v0.0.1-alpha.3]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.2...0.0.1-alpha.3
