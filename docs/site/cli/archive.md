# archive

Move entries to the Archive section.

## Usage

```bash
doing archive [OPTIONS]
```

## Aliases

- `move`

## Options

| Flag | Description |
| --- | --- |
| `--bool BOOL_OP` | Boolean operator for multiple filters (AND, OR, NOT) |
| `--case` | Case-sensitive search |
| `--count COUNT` | Number of entries to archive |
| `-x, --exact` | Match search term exactly |
| `-i, --interactive` | Select entries interactively |
| `-k, --keep` | Keep the original entry in its current section |
| `--not` | Negate the search filter |
| `--search SEARCH` | Filter entries by search term |
| `-s, --section SECTION` | Section to archive from |
| `--tag TAG` | Filter entries by tag |
| `--to TO_SECTION` | Destination section (default: Archive) |
| `--unfinished` | Archive only unfinished entries |
| `--val VALUE` | Filter by tag value |

## Examples

Archive all entries from the Currently section:

```bash
doing archive -s Currently
```

Move entries matching a tag to a specific section:

```bash
doing archive --tag project --to "Completed Projects"
```

Interactively select entries to archive:

```bash
doing archive -i
```
