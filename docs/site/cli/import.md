# import

Import entries from other sources.

## Usage

```bash
doing import [OPTIONS] [FILE]
```

## Arguments

| Argument | Description |
| --- | --- |
| `FILE` | File to import entries from |

## Options

| Flag | Description |
| --- | --- |
| `--from FORMAT` | Source format to import from |
| `-s, --section SECTION` | Section to import entries into |
| `--prefix PREFIX` | Prefix to add to imported entry titles |
| `--tag TAG` | Tag to add to all imported entries |
| `--autotag` | Automatically generate tags from entry content |

## Examples

Import entries from a file:

```bash
doing import tasks.csv --from csv
```

Import into a specific section with a tag:

```bash
doing import export.json --from json -s "Imported" --tag imported
```

Import with a prefix and autotag:

```bash
doing import backup.doing --prefix "[old]" --autotag
```
