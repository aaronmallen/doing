---
id: 0002
title: Hyphenated Command Names
status: active
scope: [cli]
tags: [cli, commands, naming]
created: 2026-03-21
resolved:
---

# DEV-0002: Hyphenated Command Names

## Summary

Multi-word commands use hyphens instead of underscores.

## Scope

All multi-word command and subcommand names.

## Original Behavior

Brett's `doing` uses underscores for multi-word commands:

```sh
doing last_note
doing mark_flagged
doing show_sections
```

## Our Behavior

We use hyphens for multi-word commands:

```sh
doing last-note
doing mark-flagged
doing show-sections
```

## Rationale

Hyphens are the convention used by the vast majority of modern CLI tools (`git`, `cargo`, `rustup`, `gh`, `docker`,
etc.). Following this convention makes `doing` feel more natural alongside other tools and avoids the need to remember
which convention a particular tool uses.

## Migration

Replace underscores with hyphens in any scripts or aliases that call multi-word `doing` commands.

```sh
# Before
doing last_note

# After
doing last-note
```

## References

- [POSIX Utility Conventions](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html)
