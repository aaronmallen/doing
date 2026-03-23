---
id: 0011
title: --age has no short flag (-a conflicts with tag --autotag)
scope: [cli]
tags: [cli]
created: 2026-03-23
---

# DEV-0011: `--age` Has No `-a` Short Flag

## Ruby Behavior

`doing show --help` shows `-a, --age=AGE`.

## Our Behavior

`--age` on `FilterArgs` has no short flag.

## Rationale

`FilterArgs` is a shared struct flattened into many commands. The `tag` command has `-a` for
`--autotag`, which conflicts with `-a` on `FilterArgs.age`. Since clap does not allow per-command
overrides of flattened struct short flags, `-a` cannot be added to `FilterArgs` without breaking
`tag`. The `--age` flag is long-form only.
