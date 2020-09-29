# Developing jsonpath-reference-implementation

This guide describes how to set up a development environment and run the tests.
If you'd like to submit changes, see the [Contributor Guide](./CONTRIBUTING.md).

## Setting up a development environment

The reference implementation is written in [Rust](https://www.rust-lang.org/).

### Installing Rust

Install the latest stable version of Rust by following [these instructions](https://rustup.rs/).
This will also install [Cargo](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html) which is Rust's
build system and package manager.

If you are new to Rust, check out the resources at [Learn Rust](https://www.rust-lang.org/learn).

## Running the Compliance Test Suite

Once Rust is installed, run the Compliance Test Suite on the command line by changing directory to
a clone of this repository and issuing:
```
cargo test
```

If all goes well, the output should include this:
```
...
test tests::compliance_test_suite ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
...
```

## Debugging test failures

To run a test on its own, edit [cts.json](tests/cts.json) and set the `focus` property of the relevant
test to `true`, for example:
<pre>
  }, {
    "name": "wildcarded child",
    <b>"focus": true,</b>
    "selector": "$.*",
    "document": {"a" : "A", "b" : "B"},
    "result": ["A", "B"]
  }, {
</pre>

When one or more tests are focussed in this way, the test suite will fail with the message
"testcase(s) still focussed" even if all the tests pass.
This prevents pull requests being merged in which tests are accidentally left focussed.

To see details of which tests run, use:
```
cargo test -- --show-output
```

If you want a bit more detail of what Cargo is doing, use:
```
cargo test -v
```

## Editing

First class Rust support is available in various editors. See [Rust Tools](https://www.rust-lang.org/tools)
for details.

## Code formatting and linting

To format the code, issue:
```
cargo fmt
```

To check the code for stylistic and other problems (known as "linting"), issue:
```
cargo clippy
```

The following one-liner formats/lints the code and, if that was successful, runs the tests:
```
cargo fmt && cargo clippy && cargo test
```

## Generating the docs

To generate and view the docs, issue:
```
cargo doc --document-private-items --open
```
