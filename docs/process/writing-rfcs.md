# Requests for Comments (RFCs)

## Creating an RFC

Create an issue using the template below.

## When to Use an RFC

RFCs are appropriate for:

- New features that affect the public API
- Significant changes to existing behavior
- Cross-cutting concerns that affect multiple parts of the codebase
- Changes that benefit from discussion before implementation

Unlike ADRs which document decisions already made, RFCs gather feedback and build consensus before committing to an
approach.

## Template

```markdown
# RFC: Title

## Summary

One paragraph explaining the proposed change.

## Motivation

Why are we doing this? What problem does it solve?

## Goals

- Goal 1
- Goal 2

## Non-Goals

- What this RFC explicitly does not aim to address

## Proposed Design

Detailed explanation of the proposed design. Include:

- API changes or new APIs
- Data structures
- Algorithms
- Code examples where helpful

## Alternatives Considered

What other designs were considered and why were they not chosen?

## Unresolved Questions

- Questions that need to be answered before or during implementation

## Future Possibilities

Things that could build on this RFC but are out of scope for now.

## References

- Related RFCs, ADRs, issues, or external resources
```
