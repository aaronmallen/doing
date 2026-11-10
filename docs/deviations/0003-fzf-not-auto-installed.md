---
id: 0003
title: fzf Not Auto-Installed
scope: [cli]
tags: [cli, fzf, dependencies]
created: 2026-03-21
---

# DEV-0003: fzf Not Auto-Installed

## Summary

We do not automatically install fzf; a built-in selection menu is used when fzf is unavailable.

## Scope

Any command that presents an interactive selection menu (e.g., `doing select`, `doing tag --interactive`).

## Original Behavior

Brett's `doing` will automatically install [fzf](https://github.com/junegunn/fzf) if it is not already present on the
system. Once installed, fzf is used for all interactive selection prompts.

## Our Behavior

We never install software on the user's behalf. If fzf is available on `$PATH`, it will be used for interactive
selection. If it is not available, a built-in selection menu is used instead.

## Rationale

Silently installing third-party software is surprising behavior for a CLI tool and can conflict with system package
managers, corporate policies, or user preferences. Users should have explicit control over what gets installed on their
system.

## Migration

No action required. If you want fzf-powered selection, install it yourself:

```sh
# macOS
brew install fzf

# Debian/Ubuntu
sudo apt install fzf

# From source
git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf && ~/.fzf/install
```

If fzf is not installed, the built-in menu will be used automatically.

## References

- [fzf](https://github.com/junegunn/fzf)
