# note

Add or display notes on the last entry.

## Usage

```bash
doing note [OPTIONS] [TEXT]...
```

## Arguments

| Argument | Description |
| --- | --- |
| `TEXT` | Note text to append |

## Options

| Flag | Description |
| --- | --- |
| `--ask` | Prompt interactively for a note |
| `--count COUNT` | Maximum number of entries to annotate |
| `-e, --editor` | Open an editor to compose the note |
| `-i, --interactive` | Interactively select entries to annotate |
| `-n, --note NOTE_TEXT` | Note text to append (can be repeated for multiple lines) |
| `-r, --remove` | Remove all notes from the entry |
| `--after AFTER` | Date range (e.g. "monday to friday") |
| `--age AGE` | Which end to keep when limiting by count (newest/oldest) |
| `--before BEFORE` | Upper bound for entry date |
| `--bool BOOL_OP` | Boolean operator for combining tag filters (and/or/not/pattern) |
| `--case CASE` | Case sensitivity for search (smart/sensitive/ignore) |
| `-x, --exact` | Use exact (literal substring) matching for search |
| `--from FROM` | Date range expression (e.g. "monday to friday") |
| `--not` | Negate all filter results |
| `--only-timed` | Only include entries with a recorded time interval |
| `--search SEARCH` | Text search query |
| `-s, --section SECTION` | Section name to filter by |
| `--tag TAG` | Tags to filter by (can be repeated) |
| `-u, --unfinished` | Only include unfinished entries (no @done tag) |
| `--val VAL` | Tag value queries (e.g. "progress > 50") |

## Examples

Add a note to the last entry:

```bash
doing note "Found the root cause in the auth module"
```

Add a note using the editor:

```bash
doing note -e
```

Remove notes from the last entry interactively:

```bash
doing note -r -i
```
