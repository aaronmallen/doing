---
id: 0012
title: redo --file has no short flag (-f conflicts with global --doing-file)
scope: [redo]
tags: [cli]
created: 2026-03-23
---

# DEV-0012: `redo --file` Has No `-f` Short Flag

## Ruby Behavior

`doing redo --help` shows `-f, --file=PATH`.

## Our Behavior

`redo --file` has no short flag.

## Rationale

The global `--doing-file` flag uses `-f` (lowercase). Clap does not allow per-command overrides
of global short flags, so `-f` cannot be reused for `--file` on `redo` without a conflict.
