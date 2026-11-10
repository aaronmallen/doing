---
id: 0006
title: update command is self-update, not entry update
scope: [update]
tags: [cli]
created: 2026-03-22
---

# DEV-0006: `update` Command Is Self-Update

## Ruby Behavior

`doing update` updates the doing gem itself (`gem update doing`).

## Our Behavior

Our `doing update` is a self-update command for the Rust binary, distributed through different channels.
The implementation and behavior differ from Ruby doing's gem-based update mechanism.

## Rationale

The update mechanism is inherently tied to the distribution method. Ruby doing updates via RubyGems;
our binary has its own update path. The command name is the same but the underlying behavior is
distribution-specific by nature.

## Affected Tests

Tests expecting Ruby doing's `update` behavior are marked `#[ignore]` with a reference to this deviation.
