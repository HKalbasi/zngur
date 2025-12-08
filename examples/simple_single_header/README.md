# Simple Single Header Example

This example demonstrates the `--single-header` flag, which generates a single `generated.h` file instead of splitting into `generated.h` and `zngur.h`. This emulates the old behavior of zngur before the header split.

## Differences from the regular simple example

- Uses `--single-header` flag in the Makefile
- Does not require `-o` flag (output directory) since utility headers are merged into the main header
- Does not need `-I.` flag for compilation since there's no separate `zngur.h` to include
- The `.gitignore` doesn't include `zngur.h` since it's not generated

## Building

```bash
make
```

## Running

```bash
./a.out
```

## When to use single-header mode

Use `--single-header` when:

- You want simpler build configuration (no need to manage include paths for utility headers)
- You're migrating from an older version of zngur
- You have a simple project that doesn't need the modularity of split headers

Use split headers (default) when:

- You have multiple zngur-generated libraries that can share the same `zngur.h`
- You want faster incremental compilation (changes to your types don't require recompiling the infrastructure)
- You want cleaner separation between infrastructure and user code
