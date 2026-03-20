# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog],
and this project adheres to [Break Versioning].

## [Unreleased]

### Fixed

- Default template was empty, causing `show` and other display commands to produce no output without explicit template
  config
- Default template now includes colors (`%boldwhite`, `%boldcyan`, `%cyan`) matching the Ruby doing gem style
- [#24] `reset` command crashes with `--search`, `--back`, or `--from` flags when search config is not explicitly set

## 0.0.1-alpha.1 - 2026-03-19

Initial alpha release

[Break Versioning]: https://www.taoensso.com/break-versioning
[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/

[#24]: https://github.com/aaronmallen/doing/issues/24

[Unreleased]: https://github.com/aaronmallen/doing/compare/0.0.1-alpha.1...main
