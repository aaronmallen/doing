---
name: format
description: Format and lint the project, auditing changed Rust files against code style guidelines.
---

# Format

Format and lint the project, auditing changed Rust files against code style guidelines.

## Instructions

### 1. Run Formatter

Run `mise run format` to format code. Fix any **errors** the formatter was unable to address.

### 2. Audit Changed Files

```sh
!if [ -d .jj ]; then jj show; else git diff; fi
```

For each `.rs` file in the diff, spin up a **separate agent** to audit that file against `docs/dev/code-style.md`.
Launch all agents in parallel.

Each agent should read the full file and check module-level ordering (constants first, then type groups
alphabetically with impl blocks immediately following their type, then free functions) and report any
violations with line numbers.

### 3. Fix Violations

Fix any violations the agents report.

### 4. Re-lint

Run `mise run lint` again after fixes to ensure formatting is still clean.

### 5. Run Tests

Run `mise run test` to confirm nothing is broken.
