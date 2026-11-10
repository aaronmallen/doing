---
id: 0002
title: Cargo Workspace Migration
status: active
tags: [architecture, build]
created: 2026-03-23
superseded-by:
---

# ADR-0002: Cargo Workspace Migration

## Status

![Static Badge](https://img.shields.io/badge/Active-green?style=for-the-badge)

## Summary

Convert the `doing` codebase from a single Cargo crate to a Cargo workspace containing 8 crates under `crates/`,
enforcing the existing module dependency graph at compile time, enabling incremental compilation, and making the core
libraries independently publishable.

## Context

`doing` is a Rust CLI tool for time tracking. As of alpha, it is a single crate with well-defined modules (`errors`,
`time`, `taskpaper`, `config`, `paths`, `ops`, `template`, `plugins`, `cli`). These modules already have clean, stable
boundaries — no module reaches across the dependency graph to import from a module that logically sits above it.

However, because everything lives in one crate:

- There is no compile-time enforcement that e.g. `config` does not accidentally import from `taskpaper`.
- Any change to a leaf module causes the entire crate to recompile.
- The library surface area cannot be carved up for external consumers who might want only `doing-taskpaper` or
  `doing-time` without pulling in CLI or plugin dependencies.

## Decision

Convert to a Cargo workspace. The root `Cargo.toml` becomes the workspace manifest. All library code moves under
`crates/`. The binary entry point (`main.rs` and `cli/`) stays in the root crate, which becomes a thin binary crate
depending on all workspace library crates.

### Crate Inventory

| Crate             | Responsibility                                          | Internal Dependencies                                                            |
|-------------------|---------------------------------------------------------|----------------------------------------------------------------------------------|
| `doing-error`     | Shared error types                                      | —                                                                                |
| `doing-time`      | Time parsing and formatting                             | `doing-error`                                                                    |
| `doing-taskpaper` | TaskPaper document model                                | `doing-error`, `doing-time`                                                      |
| `doing-config`    | Configuration loading, types, env vars, path utilities  | `doing-error`                                                                    |
| `doing-ops`       | Core operations (filter, search, autotag, backup, undo) | `doing-error`, `doing-time`, `doing-taskpaper`, `doing-config`                   |
| `doing-template`  | Template rendering and output formatting                | `doing-error`, `doing-time`, `doing-taskpaper`, `doing-config`                   |
| `doing-plugins`   | Export and import plugin system                         | `doing-error`, `doing-time`, `doing-taskpaper`, `doing-config`, `doing-template` |
| `doing` (root)    | Binary crate: CLI layer only                            | all workspace crates                                                             |

### Dependency Graph

```text
doing-error                     (leaf)
  ├── doing-time                (→ doing-error)
  │     └── doing-taskpaper     (→ doing-error, doing-time)
  └── doing-config              (→ doing-error)
        ├── doing-ops           (→ doing-error, doing-time, doing-taskpaper, doing-config)
        ├── doing-template      (→ doing-error, doing-time, doing-taskpaper, doing-config)
        │     └── doing-plugins (→ doing-error, doing-time, doing-taskpaper, doing-config, doing-template)
        └── doing (binary)      (→ all workspace crates)
```

### Workspace Layout

```text
Cargo.toml               ← workspace manifest + [workspace.dependencies]
crates/
  error/Cargo.toml        ← name = "doing-error"
  time/Cargo.toml         ← name = "doing-time"
  taskpaper/Cargo.toml    ← name = "doing-taskpaper"
  config/Cargo.toml       ← name = "doing-config"
  ops/Cargo.toml          ← name = "doing-ops"
  template/Cargo.toml     ← name = "doing-template"
  plugins/Cargo.toml      ← name = "doing-plugins"
src/
  main.rs                 ← binary entry point (unchanged)
  cli/                    ← CLI layer (unchanged)
```

### Implementation Principles

- **Bottom-up extraction** — extract crates in dependency-graph order so that each new crate depends only on crates
  that already exist.
- **Green at every step** — `mise run format lint test` must pass after each extraction before the next begins.
- **One crate per PR** — each extraction is a focused, reviewable unit of work.
- **Minimal visibility promotion** — items that are `pub(crate)` and accessed across the new crate boundary must become
  `pub`, but prefer restructuring over blanket promotion where possible.
- **Workspace dependency centralization** — `[workspace.dependencies]` declares all dependency versions. No member crate
  specifies a version or path directly in its own `[dependencies]` table.

### Extraction Order

1. Initialize workspace + extract `doing-error`
2. Extract `doing-time` (blocked by step 1)
3. Extract `doing-config` (blocked by step 1; independent of step 2)
4. Extract `doing-taskpaper` (blocked by steps 1–2)
5. Extract `doing-ops` (blocked by steps 3–4)
6. Extract `doing-template` (blocked by steps 3–4; independent of step 5)
7. Extract `doing-plugins` (blocked by step 6)
8. Finalize binary crate (blocked by step 7)

## Dependencies

| Crate | Version | Purpose | License |
|-------|---------|---------|---------|
| -     | -       | -       | -       |

## Consequences

### Positive

- **Compile-time boundary enforcement** — the Rust compiler rejects imports that violate the dependency graph.
- **Incremental compilation** — a change to a CLI command does not recompile the TaskPaper parser or time utilities.
- **Parallel compilation** — Cargo compiles independent crates concurrently (`doing-taskpaper` and `doing-config` build
  in parallel; `doing-ops` and `doing-template` build in parallel).
- **Reusable libraries** — `doing-taskpaper`, `doing-time`, and `doing-config` can be published independently.
- **Focused responsibility** — each crate has a small, well-defined public API surface.

### Negative

- **Migration effort** — 8 extraction steps, each requiring import rewrites and `Cargo.toml` additions.
- **Workspace boilerplate** — each crate requires its own `Cargo.toml` and `lib.rs`.
- **Visibility promotion** — some `pub(crate)` items must become `pub` to cross crate boundaries.
- **Test organization** — unit tests move with their source files into the new crates. Integration tests under
  `tests/integration/` are unaffected since they invoke the compiled binary.

## Open Questions

- Should `doing-taskpaper` re-export `doing-time` types that appear in its public API, or should callers depend on
  `doing-time` directly?
- At what point should workspace crates be published to crates.io independently?

## Future Work

- Per-crate changelogs once the API surface stabilizes.
- Investigate whether `doing-taskpaper` is general enough to deserve its own repository.
- Benchmark compile times before and after to quantify the incremental compilation benefit.

## References

- [Discussion #213: Cargo Workspace with Multiple Crates](https://github.com/aaronmallen/doing/discussions/213)
- [ADR-0001: Integration Test Structure][0001]
- [Cargo Workspaces — The Cargo Book](https://doc.rust-lang.org/cargo/reference/workspaces.html)

[0001]: 0001-integration-test-structure.md
