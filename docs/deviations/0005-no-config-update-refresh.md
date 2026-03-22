---
id: 0005
title: No config update/refresh subcommand
scope: [config]
tags: [cli, config]
created: 2026-03-22
---

# DEV-0005: No `config update/refresh` Subcommand

## Ruby Behavior

`doing config update` (aliased as `refresh`) re-reads the default config template and adds any missing
keys to the user's config file. This serves as a migration tool when new config options are added in a
new version of Ruby doing.

## Our Behavior

We do not implement `config update` or `config refresh`.

## Rationale

Our TOML configuration uses serde defaults — missing keys are automatically filled with default values
at deserialization time. There is no need for an explicit migration step. The config file only contains
values the user has explicitly set; everything else falls back to compiled-in defaults.

## Affected Tests

Tests for `config update`/`config refresh` are marked `#[ignore]` with a reference to this deviation.
