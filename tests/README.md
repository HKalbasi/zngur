# End-to-end tests

Beside the examples (which must compile and stay advertised), these tests cover
cases that should be rejected and patterns we don't want in the examples.

Run them with:

```sh
cargo xtask test       # check
cargo xtask test --fix # update the fail snapshots
```

Tests are grouped by feature area:

- `parser/`: parsing and validation of a single `.zng` file.
- `merge/`: importing and merging multiple `.zng` files.
- `cfg/`: conditional compilation (`#if` / `#match`).
- `codegen/`: generating a bridge and running it.
- `deprecated/`: deprecated items still work but warn.
- `cli/`: the `zngur` command-line interface (free-form Nushell tests).

Each test is a `*.zng` file whose first `//@` directive selects the mode. The CLI
always runs in the test's directory, so spans point at the file name.

- `//@ fail`: `zngur generate` must reject it. The diagnostics, including the
  source spans, are compared with the sibling `*.stderr` snapshot.
- `//@ pass`: the sibling `*.cpp` is linked into a binary built from the sibling
  `*.rs`, and the binary must run successfully. The `*.rs` is the crate root and
  must declare the generated bridge (`mod generated;`). If generating the bridge
  emits warnings (e.g. from a deprecated item), they are compared with the sibling
  `*.stderr` snapshot, which must exist; create it with `cargo xtask test --fix`.
  The `.cpp`'s directory is on the include path, so it can ship headers included
  via `#cpp_additional_includes`.

Files whose name starts with `_` are fixtures, not tests. Use them for the files
that a test imports with `merge`.

## Free-form (Nushell) tests

Anything that doesn't fit a single `.zng` file — most notably CLI behavior — is
written as a directory containing a `main.nu`. The harness copies the directory
into a scratch dir under `target/nu-tests/`, copies `tests/snapshot-utils.nu`
alongside it, and runs `nu main.nu` there, so the script may create files freely.
The script must exit `0`, or the status given by a `#@ run-status: <n>` directive
at the top of `main.nu`.

The script gets:

- `$env.ZNGUR_BIN`: the `zngur` binary to test (its directory is on `PATH`).
- `$env.ZNGUR_TEST_DIR`: the scratch directory (the current directory).
- `$env.ZNGUR_TEST_SOURCE_DIR`: the test's directory in the repository, where
  snapshot files live.
- `$env.ZNGUR_UPDATE_SNAPSHOTS`: set by `cargo xtask test --fix`.

Use `use ./snapshot-utils.nu *` and `assert-snapshot <name> <actual> <path>` to
compare output with a snapshot.

The `cli/` directory holds the CLI's own free-form tests.
