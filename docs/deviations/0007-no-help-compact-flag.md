---
id: 0007
title: No help -c compact flag
scope: [help]
tags: [cli]
created: 2026-03-22
---

# DEV-0007: No `help -c` Compact Flag

## Ruby Behavior

`doing help -c` displays a compact command listing.

## Our Behavior

We use clap's built-in help system, which does not support a compact mode flag.

## Rationale

Clap provides standardized help output. Adding a custom compact mode would require fighting clap's
output generation for marginal benefit. Users can pipe through `grep` or other tools for filtered output.

## Affected Tests

Tests for `help -c` are marked `#[ignore]` with a reference to this deviation.
