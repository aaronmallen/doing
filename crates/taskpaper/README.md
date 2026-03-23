# doing-taskpaper

TaskPaper document parser and serializer for the [doing](https://github.com/aaronmallen/doing) CLI.

This crate provides the document model for doing files: sections, entries with timestamps and tags, notes, and the
parser/serializer that round-trips the on-disk format. It also handles atomic file I/O for doing files.

## Usage

```rust
use doing_taskpaper::{Document, Entry, Section, Tags, Note};

// Parse a doing file
let content = std::fs::read_to_string("what_was_i_doing.md").unwrap();
let doc = Document::parse(&content);

for section in doc.sections() {
    println!("{}:", section.title());
    for entry in section.entries() {
        println!("  {} - {}", entry.date().format("%H:%M"), entry.title());
    }
}
```

## License

MIT — see [LICENSE](../../LICENSE) for details.
