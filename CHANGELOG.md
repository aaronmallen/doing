# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Break Versioning].

## [Unreleased]

### Fixed

- [#66] `--at` flag resolves bare times ("2pm", "3:30pm") to yesterday instead of today

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

[#18]: https://github.com/aaronmallen/doing/issues/18
[#19]: https://github.com/aaronmallen/doing/issues/19
[#20]: https://github.com/aaronmallen/doing/issues/20
[#21]: https://github.com/aaronmallen/doing/issues/21
[#23]: https://github.com/aaronmallen/doing/issues/23
[#24]: https://github.com/aaronmallen/doing/issues/24
[#26]: https://github.com/aaronmallen/doing/issues/26
[#33]: https://github.com/aaronmallen/doing/issues/33
[#44]: https://github.com/aaronmallen/doing/issues/44
[#58]: https://github.com/aaronmallen/doing/issues/58
[#66]: https://github.com/aaronmallen/doing/issues/66

[Unreleased]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.2...main
[v0.0.1-alpha.2]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.1...0.0.1-alpha.2
