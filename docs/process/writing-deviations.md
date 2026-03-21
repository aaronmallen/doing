# Deviation Records

## Creating a Deviation Record

1. Create a new file in `docs/deviations/` with the next available ID (e.g., `0001-my-deviation.md`)
2. Fill in all sections using the template below
3. Add an entry to the [deviation index][deviation-index]
4. Submit as part of your pull request

## When to Write a Deviation Record

Write a deviation record when this project's behavior intentionally differs from Brett Terpstra's [doing]. Deviations
may include:

- Commands with different syntax or argument handling
- Output format changes that could break existing scripts
- Features that are not supported or deliberately omitted
- Default values or configuration that differ from the original
- Behavior changes in how entries, tags, or sections are managed

Every user-visible difference from the original CLI should have a corresponding deviation record so that users migrating
from Brett's `doing` can understand what changed and why.

## Deviation Structure

Each deviation record describes:

- **Scope**: Which command, flag, or feature area is affected
- **Original behavior**: How Brett's `doing` handles it
- **Our behavior**: What this project does instead
- **Rationale**: Why we chose to deviate
- **Migration**: How users coming from the original can adapt

## Template

```markdown
---
id: 0000
title: Deviation Title
scope: []
tags: []
created: YYYY-MM-DD
---

# DEV-0000: Title

## Summary

One sentence describing the deviation.

## Scope

Which command(s), flag(s), or feature area(s) are affected.

## Original Behavior

How Brett Terpstra's `doing` handles this. Include examples where helpful.

## Our Behavior

What this project does instead. Include examples where helpful.

## Rationale

Why we chose to deviate from the original behavior.

## Migration

How users coming from the original `doing` can adapt their workflows or scripts.

## References

- Related ADRs, issues, or external resources
```

[deviation-index]: https://github.com/aaronmallen/doing/blob/main/docs/deviations/README.md
[doing]: https://github.com/ttscoff/doing
