# Testing

This guide covers how to write and organize tests in Doing. Tests live alongside the code they test, in a `#[cfg(test)]`
module at the bottom of each source file.

The goal is to test meaningful behavior without over-testing trivial code. When in doubt, focus on logic that could
break—transformations, edge cases, and complex calculations.

## Running Tests

```bash
mise run test                  # Run all tests with coverage
mise run test -- --filter xyz  # Run tests matching "xyz"

# Without mise:
./tasks/test/build
```

## What to Test

**Test:**

- Methods with logic (arithmetic, transformations, conditionals)
- Display/formatting implementations
- Custom `PartialEq`/`PartialOrd` implementations
- Edge cases and boundary conditions
- Inverse operations (roundtrip tests)

**Skip:**

- Simple constructors that just assign fields
- Trivial getters that return field values
- `From` implementations that only call `Self::new()`

Before writing a test, ask: "Does this test verify actual logic, or just that Rust's field assignment works?"

## Test Structure

Tests are organized as nested modules within each source file:

```rust
#[cfg(test)]
mod test {
    use super::*;

    mod method_name {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn it_returns_expected_value() {
            let input = SomeType::new(42);

            assert_eq!(input.value(), 42);
        }
    }
}
```

## Conventions

**Naming:** Test functions use the pattern `it_<does_something>`. Module names match the method being tested.

**Ordering:** Test modules follow [code style][code-style] ordering - associated functions first (alphabetically),
then methods (alphabetically).

**Test body structure:** Separate setup from assertions with a blank line. For tests with multiple assertion groups,
separate each group with a blank line:

```rust
#[test]
fn it_handles_multiple_cases() {
    let foo = "foo";
    let bar = "bar";

    assert_ne!(foo, bar);

    let baz = "baz";

    assert_ne!(foo, baz);
}
```

**Assertions:** Use `pretty_assertions` for `assert_eq!` and `assert_ne!`:

```rust
use pretty_assertions::assert_eq;
```

[code-style]: https://github.com/aaronmallen/doing/blob/main/docs/dev/code-style.md
