---
id: 0013
title: completion generate --file has no short flag (-f conflicts with global --doing-file)
scope: [completion]
tags: [cli]
created: 2026-03-23
---

# DEV-0013: `completion generate --file` Has No `-f` Short Flag

## Ruby Behavior

`doing completion generate --help` shows `-f, --file=PATH`.

## Our Behavior

`completion generate --file` has no short flag.

## Rationale

The global `--doing-file` flag uses `-f` (lowercase). Clap does not allow per-command overrides
of global short flags, so `-f` cannot be reused for `--file` on `completion generate` without a
conflict. This is the same limitation documented in DEV-0012 for `redo --file`.
