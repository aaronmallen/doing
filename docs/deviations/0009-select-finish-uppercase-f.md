---
id: 0009
title: select uses -F (uppercase) for --finish instead of -f
scope: [select]
tags: [cli]
created: 2026-03-23
---

# DEV-0009: `select` Uses `-F` for `--finish` Instead of `-f`

## Ruby Behavior

`doing select -f` maps to `--finish`.

## Our Behavior

We use `-F` (uppercase) for `--finish` on the `select` command.

## Rationale

The global `--doing-file` flag uses `-f` (lowercase) via clap's global flag mechanism. Clap does not
allow per-command overrides of global short flags, so `-f` cannot be reused for `--finish` on `select`
without a conflict. Uppercase `-F` is the closest alternative.
