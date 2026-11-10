---
id: 0008
title: select uses --no-menu instead of --force
scope: [select]
tags: [cli]
created: 2026-03-22
---

# DEV-0008: `select` Uses `--no-menu` Instead of `--force`

## Ruby Behavior

`doing select --force` skips confirmation prompts.

## Our Behavior

We use `--no-menu` for non-interactive batch mode on the `select` command. There is no `--force` flag.

## Rationale

`--no-menu` more clearly describes what the flag does (skip the interactive menu) compared to the
overloaded `--force` which has different meanings across CLI tools. This is a deliberate naming choice.

## Affected Tests

Tests for `select --force` are marked `#[ignore]` with a reference to this deviation.
