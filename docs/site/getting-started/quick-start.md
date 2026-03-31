# Quick Start

This page walks through the basic `doing` workflow: adding entries, checking what you've been working
on, and finishing tasks.

## Add an Entry

Use `doing now` to record what you are currently working on:

```sh
doing now "Writing API integration tests"
```

The entry is added to the **Currently** section of your doing file with the current timestamp.

You can add tags inline to categorize your work:

```sh
doing now "Reviewing pull request for auth module @code-review"
```

## Check Recent Entries

See your most recent entry with `doing last`:

```sh
doing last
```

View several recent entries with `doing recent`:

```sh
doing recent 5
```

## Finish a Task

When you are done with something, mark it as finished. `doing finish` moves your most recent entry to
the **Archive** section and tags it with `@done` and a timestamp:

```sh
doing finish
```

To finish a specific entry interactively:

```sh
doing finish --choose
```

## Add a Completed Item

If you forgot to track something while you were doing it, use `doing done` to add an entry that is
already marked finished:

```sh
doing done "Fixed the login redirect bug"
```

You can backdate it with a natural-language time expression:

```sh
doing done "Deployed v2.3.1 to staging" --back 30m
```

## Review Your Day

See everything you have worked on today:

```sh
doing today
```

Or check what you did yesterday:

```sh
doing yesterday
```

## Next Steps

- Learn about [Core Concepts](./concepts) to understand how `doing` organizes your data.
- Run `doing --help` or `doing <command> --help` to explore all available commands.
