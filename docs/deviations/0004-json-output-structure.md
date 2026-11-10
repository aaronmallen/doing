---
id: 0004
title: JSON Output Structure
scope: [show, output]
tags: [cli, json, output, formats]
created: 2026-03-21
---

# DEV-0004: JSON Output Structure

## Summary

JSON output uses a top-level array of section objects instead of a single section object.

## Scope

The `--output json` flag on `show` and other display commands.

## Original Behavior

Brett's `doing` outputs a single JSON object with `section`, `items`, and `timers` keys:

```json
{
  "section": "Currently",
  "items": [...],
  "timers": [...]
}
```

For an empty file, it outputs:

```json
{"section":"","items":[],"timers":""}
```

## Our Behavior

We output a top-level array of section objects:

```json
[
  {
    "section": "Currently",
    "items": [...]
  }
]
```

For an empty file, we output an empty array:

```json
[]
```

## Rationale

A top-level array naturally supports multi-section output (e.g. `doing show all`) without requiring a different
structure for single-section vs. multi-section queries. The original's single-object format only represents one section
at a time, which limits composability.

## Migration

Scripts that access `json.section` or `json.items` directly should index into the array first:

```sh
# Before (Ruby doing)
doing show --output json | jq '.items[]'

# After
doing show --output json | jq '.[].items[]'
```

## References

- [Ruby doing JSON export plugin](https://github.com/ttscoff/doing/blob/main/lib/doing/plugins/export/json_export.rb)
