# rotate

Move entries to a dated archive file.

## Usage

```bash
doing rotate [OPTIONS]
```

## Options

| Flag | Description |
| --- | --- |
| `--bool BOOL_OP` | Boolean operator for multiple filters (AND, OR, NOT) |
| `--case` | Case-sensitive search |
| `--count COUNT` | Number of entries to rotate |
| `-x, --exact` | Match search term exactly |
| `-k, --keep` | Keep the original entry in its current section |
| `--not` | Negate the search filter |
| `--search SEARCH` | Filter entries by search term |
| `-s, --section SECTION` | Section to rotate from |
| `--tag TAG` | Filter entries by tag |
| `--before BEFORE` | Only rotate entries before a given date |
| `--val VALUE` | Filter by tag value |

## Examples

Rotate all entries older than 30 days:

```bash
doing rotate --before "30 days ago"
```

Rotate entries from a specific section:

```bash
doing rotate -s Currently --count 10
```

Rotate entries matching a tag:

```bash
doing rotate --tag meeting --before "last week"
```
