---
id: 0001
title: Integration Test Structure
status: active
tags: [testing, conventions]
created: 2026-03-21
---

# ADR-0001: Integration Test Structure

## Status

![Static Badge](https://img.shields.io/badge/Active-green?style=for-the-badge)

## Summary

Establishes a consistent directory layout, module hierarchy, and naming conventions for the integration test suite under
`tests/integration/`.

## Context

As the number of CLI subcommands and cross-cutting behaviors grows, the integration test suite needs a predictable
structure so that contributors can quickly find existing tests, add new ones in the right place, and read test output
that clearly communicates what is being tested. Without conventions, tests tend to accumulate in large files with
inconsistent naming, making the suite harder to navigate and maintain.

## Decision

### Directory Layout

```text
tests/integration/
  main.rs
  commands.rs
  commands/
    <command>.rs
    <command>/
      when_<context>.rs
  behavior.rs
  behavior/
    <concern>.rs
    <concern>/
      when_<context>.rs
  support.rs
  support/
    helpers.rs
```

All modules use the `file.rs` + `file/` directory pattern. Never use `mod.rs`.

### Module Hierarchy

`main.rs` declares three top-level modules:

```rust
mod commands;
mod behavior;
mod support;
```

**`commands.rs`** declares one submodule per CLI subcommand.

**`behavior.rs`** declares one submodule per cross-cutting concern (e.g., `chronify`, `file_sort`, `global_flags`,
`interactive`, `output`, `smoke`, `template`).

**`support.rs`** declares shared test infrastructure:

```rust
pub(crate) mod helpers;
```

### Command and Behavior Module Structure

Each command module (`commands/<command>.rs`) declares `when_*` submodules that group tests by precondition or context.
The command module file itself contains only `mod` declarations and shared `use` statements. Behavior modules follow the
same pattern.

Each `when_*` file contains the actual test functions and imports what it needs directly:

```rust
use crate::support::helpers::DoingCmd;
```

Parent modules do not re-export anything. Each file is self-contained.

### Naming Conventions

**`when_*` modules** describe the precondition, state, or context under which the tests run:

- `when_back_flag_is_provided` -- a flag is passed
- `when_all_entries_are_done` -- a state exists before the command runs
- `when_back_and_took_flags_are_combined` -- multiple flags interact
- `when_search_matches_entry` -- a condition is met
- `when_doing_file_sort_is_asc` -- a config value is set

**`it_*` test functions** describe the expected behavior. They should read naturally after the `when_*` context:

> when back flag is provided, it backdates start time

Keep `it_*` names short -- the `when_*` module already provides context. Avoid repeating the context in the test name.

**Flat tests**: If a command or behavior has only 1-2 tests with no meaningful grouping, `it_*` functions can live
directly in the command module file without `when_*` submodules. Add `when_*` submodules when a third test arrives or
when there is a natural grouping.

### Test Output Format

```text
doing::integration::commands::show::when_bool_flag_is_and::it_filters_entries_by_multiple_tags
doing::integration::behavior::global_flags::when_quiet_flag_is_provided::it_suppresses_output
```

### Categorization Rules

**`commands/`**: The test exercises a specific CLI subcommand. The module is named after the subcommand.

**`behavior/`**: The test exercises a concern that spans multiple subcommands or is not tied to any single subcommand
(time parsing, file ordering, global flags, output formatting, interactive mode detection, smoke tests).

When in doubt: if the test invokes a single subcommand and asserts on that subcommand's behavior, it belongs in
`commands/`. If it tests infrastructure or cross-cutting behavior, it belongs in `behavior/`.

## Consequences

### Positive

- Test output reads as natural-language specifications (`when X, it Y`)
- Contributors can locate tests by navigating the directory tree without searching
- Adding tests for a new subcommand follows a mechanical, copy-paste-friendly pattern
- Cross-cutting concerns are separated from command-specific tests, reducing duplication

### Negative

- Deep directory nesting for commands with many flag combinations
- Small `when_*` files may feel like boilerplate for simple commands
- Renaming a subcommand requires moving both the module file and its directory

## References

- [tests/integration/](https://github.com/aaronmallen/doing/tree/main/tests/integration)
