---
id: 0010
title: --quiet has no short flag (-q used by select --query)
scope: [cli]
tags: [cli]
created: 2026-03-23
---

# DEV-0010: `--quiet` Has No `-q` Short Flag

## Ruby Behavior

`doing --help` shows `-q, --quiet` as a global flag.

## Our Behavior

`--quiet` has no short flag. `-q` is used by `select --query` instead.

## Rationale

Ruby's `doing select --help` shows `-q, --query`. The global `-q` for `--quiet` in Ruby is a GLI
framework convention that conflicts with the per-command `-q` for `--query` on `select`. Since
`--query` is used interactively and benefits more from a short flag, `-q` was assigned to `--query`
and `--quiet` is long-form only.
