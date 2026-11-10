# Core Concepts

`doing` stores everything in a single plain text file using the TaskPaper format. This page explains how
that file is structured and the key ideas behind it.

## The Doing File

By default, `doing` reads and writes a file at `~/.doing` or `~/Library/Application Support/doing/what_was_i_doing.md`
on macOS. You can change this in your configuration or pass `--doing-file` to any command.

The file is plain text in TaskPaper format, which means it is human-readable, easy to grep, and works
well with version control.

## Sections

The doing file is organized into sections. Each section is a top-level heading followed by a colon.
Two sections are used by default:

- Currently - entries you are actively working on.
- Archive - entries that have been finished.

You can create custom sections to organize your work however you like.

## Entries

An entry is a single line inside a section, indented with a tab. It consists of a timestamp and a title
separated by a pipe, optionally followed by a unique id:

```text
    - 2026-03-30 14:05 | Writing API integration tests <0123456789abcdef0123456789abcdef>
```

Entries always start with a tab and a dash (`-`). The timestamp at the front records when the entry was
created. When you write entries by hand, the 32 character id in angle brackets is optional and `doing` will
manage it for you.

## Tags

Tags are words prefixed with `@`. They can appear anywhere in the entry title. Some tags carry a value
in parentheses:

```text
    - 2026-03-30 15:30 | Deploy staging server @deploy @done(2026-03-30 15:30)
```

Common built-in tags:

| Tag                   | Purpose                         |
| --------------------- | ------------------------------- |
| `@done(timestamp)`    | When the entry was finished     |

You can use any custom tags you want, like `@meeting`, `@bugfix`, or `@project(website)`. Tags are
useful for filtering entries with commands like `doing show` and `doing tag`.

## Notes

Entries can have multi-line notes attached to them. Notes are indented lines that follow an entry, using two tabs:

```text
    - 2026-03-30 10:00 | Investigating memory leak in worker pool
        Heap profile shows growth in the connection cache.
        Suspect the idle timeout is not firing.
```

You can add notes when creating an entry with the `--note` or `-n` flag, or edit them later with
`doing note`.

## What the File Looks Like

Here is an example of a doing file on disk:

```taskpaper
Currently:
    - 2026-03-30 14:30 | Refactoring error handling in CLI commands
        Switching from anyhow to thiserror for typed errors.
    - 2026-03-30 13:00 | Writing unit tests for tag parser

Archive:
    - 2026-03-30 12:00 | Set up CI pipeline for release builds @done(2026-03-30 12:00) @project(infra)
    - 2026-03-29 17:00 | Fix off-by-one in date range query @done(2026-03-29 17:00) @bugfix
```

## Templates

Templates control how entries are displayed in output. They use a placeholder syntax to format fields
like the title, date, tags, and notes. You can inspect or edit the default template with:

```sh
doing template
```

Different commands and output formats can use different templates, letting you control exactly what
information appears and how it is formatted.

## Views

Views are named, reusable queries saved in your configuration. A view defines which section to pull
from, how many entries to show, which tags to filter by, the sort order, and which template to use
for output.

List available views:

```sh
doing views
```

Display a view:

```sh
doing view my_view
```

Views are a convenient way to avoid retyping the same flags for queries you run often.
