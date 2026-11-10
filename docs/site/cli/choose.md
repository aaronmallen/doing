# choose

Fuzzy select an entry to act on.

## Usage

```bash
doing choose [OPTIONS]
```

## Options

| Flag | Description |
| --- | --- |
| `--bool BOOL_OP` | Boolean operator for combining tag filters (and/or/not/pattern) |
| `-o, --output FORMAT` | Output format |
| `--query QUERY` | Text search query to filter entries |
| `--save-to FILE` | Save output to a file |
| `-s, --section SECTION` | Section to choose entries from |
| `--tagged TAGGED` | Filter by tags (can be repeated) |

## Examples

Open the fuzzy picker to select an entry:

```bash
doing choose
```

Choose from a specific section:

```bash
doing choose --section Archive
```

Pre-filter entries with a query before choosing:

```bash
doing choose --query "meeting" --tagged important
```
