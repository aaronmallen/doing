# Architecture Decision Records (ADRs)

## Creating an ADR

1. Create a new file in `docs/design/` with the next available ID (e.g., `0001-my-decision.md`)
2. Fill in all sections using the template below
3. Add an entry to the [ADR index][adr-index]
4. Submit as part of your pull request

## When to Write an ADR

Write an ADR when making decisions that:

- Affect the overall structure or architecture of the codebase
- Establish patterns or conventions that other code should follow
- Have long-term implications that future contributors need to understand
- Represent a significant trade-off between competing concerns

Unlike RFCs which gather feedback before committing to an approach, ADRs document decisions that have already been made.

## ADR Structure

Each ADR describes:

- **Context**: The circumstances and forces at play when the decision was made
- **Decision**: The change or approach that was chosen
- **Consequences**: The resulting effects, both positive and negative

## Status Lifecycle

| Status                           | Meaning                                                |
|----------------------------------|--------------------------------------------------------|
| ![Active][badge-active]          | Currently enforced                                     |
| ![Superseded][badge-superseded]  | Replaced by another ADR (update `superseded-by` field) |
| ![Deprecated][badge-deprecated]  | No longer followed, kept for historical reference      |

## Template

```markdown
---
id: 0000
title: ADR Title
status: active
tags: []
created: YYYY-MM-DD
superseded-by:
---

# ADR-0000: Title

## Status

![Static Badge](https://img.shields.io/badge/Active-green?style=for-the-badge)

## Summary

One paragraph explaining the decision.

## Context

Why is this decision needed? What problem does it solve?

## Decision

What we're going to do. Technical details, syntax, semantics, etc.

## Dependencies

New crate dependencies introduced by this decision.

| Crate | Version | Purpose | License |
|-------|---------|---------|---------|
| -     | -       | -       | -       |

## Consequences

### Positive

- ...

### Negative

- ...

## Open Questions

- Question 1?

## Future Work

Things explicitly out of scope, for future ADRs.

## References

- Related ADRs, discussions, external resources
```

[adr-index]: https://github.com/aaronmallen/doing/blob/main/docs/design/README.md
[badge-active]: https://img.shields.io/badge/Active-green?style=for-the-badge
[badge-deprecated]: https://img.shields.io/badge/Deprecated-red?style=for-the-badge
[badge-superseded]: https://img.shields.io/badge/XXXX--Title-black?style=for-the-badge&label=Superseded&labelColor=orange
