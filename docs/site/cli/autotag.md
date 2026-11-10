# autotag

Apply autotagging rules to existing entries.

## Usage

```bash
doing autotag [OPTIONS]
```

## Options

| Flag | Description |
| --- | --- |
| `-c, --count COUNT` | Apply to the last N entries (default: 1) |
| `-s, --section SECTION` | Section to target |

## Examples

Apply autotag rules to the last entry:

```bash
doing autotag
```

Apply autotag rules to the last 5 entries:

```bash
doing autotag --count 5
```

Apply autotag rules to all entries in a specific section:

```bash
doing autotag --section Currently --count 50
```
