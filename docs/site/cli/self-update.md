# self-update

Update doing to the latest version.

Prebuilt updates are available for Linux only. On macOS, run `cargo install doing` to update.

## Usage

```bash
doing self-update [OPTIONS]
```

## Aliases

- `update`

## Options

| Flag | Description |
| --- | --- |
| `--force` | Force reinstall even if already up to date |

## Examples

Update to the latest version:

```bash
doing self-update
```

Force a reinstall:

```bash
doing self-update --force
```
