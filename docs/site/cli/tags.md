# tags

List all tags in the doing file.

## Usage

```bash
doing tags [OPTIONS]
```

## Options

| Flag | Description |
| --- | --- |
| `--bool BOOL_OP` | Boolean operator for multiple tag filters (AND, OR, NOT) |
| `--counts` | Show the number of times each tag appears |
| `--interactive` | Select tags interactively |
| `--line` | Output tags as a comma-separated list on one line |
| `--order ORDER` | Sort order (asc, desc) |
| `--section SECTION` | Limit tags to a specific section |
| `--sort SORT` | Sort by field (name, count) |

## Examples

List all tags with counts:

```bash
doing tags --counts
```

List tags from a specific section sorted by count:

```bash
doing tags --section Currently --sort count --order desc
```

Output tags on a single line:

```bash
doing tags --line
```
