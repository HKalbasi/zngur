# Automatic Layout Resolution

Starting with Zngur 0.8.0, you can automatically determine type size and alignment at build time using `#layout(auto)` instead of manually specifying `#layout(size = X, align = Y)`.

**Note:** The intention is to eventually make `layout(auto)` the default, hopefully before 1.0.

## Requirements

- **Cargo must be installed**: `#layout(auto)` uses Cargo to locate and build your crate.
  If you're using a bare `rustc` workflow without Cargo, use explicit `#layout(size = X, align = Y)` directives instead.

## Usage

In your `.zng` file, simply use `#layout(auto)`:

```
type crate::Inventory {
    #layout(auto);
    wellknown_traits(Debug);

    fn new_empty(u32) -> crate::Inventory;
}

type std::vec::Vec<crate::Item> {
    #layout(auto);
    wellknown_traits(Debug);
}
```

That's it! Zngur will automatically extract the correct size and alignment values during the build process.

## How It Works

When you use `#layout(auto)`, Zngur:

1. Collects all types with automatic layout
2. Generates a temporary Rust program that extracts size/align for each type
3. Compiles and runs this program using your crate's artifacts
4. Caches the results for future builds
5. Uses the extracted values to generate C++ code

This happens transparently during `cargo build` when using Zngur in your build script.

### Advanced configuration in build scripts

Auto-layout works automatically with no special configuration needed in your `build.rs`.
For special cases, you can customize the layout resolution:

```rust
Zngur::from_zng_file(crate_dir.join("main.zng"))
    // ... other configuration ...
    .with_layout_cache_dir(out_dir.clone())    // Cache directory (default: OUT_DIR)
    .with_crate_path(crate_dir.clone())        // Crate path (default: CARGO_MANIFEST_DIR)
    .with_target("x86_64-unknown-linux-gnu")   // Target triple (default: auto-detect)
    .generate();
```

## Cache Invalidation

The layout cache is automatically invalidated when:

- Rust compiler version changes
- Target triple changes
- Source files are modified (detected via Cargo.lock and src/ mtime)
- Cargo features change

## CLI Command: Dumping Layouts

You can extract layouts without modifying your `.zng` file using the CLI:

```bash
zngur dump-layouts main.zng
```

This outputs the layouts in `.zng` format, ready to copy-paste:

```
# Extracted layouts for x86_64-apple-darwin (rustc 1.75.0)
# Generated at 2024-12-01 10:00:00

type crate::Inventory {
    #layout(size = 32, align = 8);
}

type std::vec::Vec<crate::Item> {
    #layout(size = 24, align = 8);
}
```

For cross-compilation:

```bash
zngur dump-layouts main.zng --target x86_64-unknown-linux-gnu
```

## Example

See [`examples/tutorial`](https://github.com/HKalbasi/zngur/blob/main/examples/tutorial) for a complete example using auto-layout.

## Troubleshooting

Auto-layout errors include helpful hints to guide you toward a solution:

### "cargo is not available"

```
error: cargo is not available
  #layout(auto) requires Cargo to extract type layouts

  = hint: install Cargo from https://rustup.rs or use explicit #layout(size = X, align = Y)
```

**Solution:** Either install Cargo via rustup, or replace `#layout(auto)` with explicit `#layout(size = X, align = Y)` directives in your `.zng` file.

### "could not find compiled library"

```
error: could not find compiled library
  Could not find compiled library for crate 'my_crate' in ./target/debug/

  = hint: run `cargo build` first to compile your crate before running zngur
```

**Solution:** Run `cargo build` to compile your crate first.

### "type not found in compiled crate"

```
error: type not found in compiled crate
  Type 'crate::MyType' not found. Check if it's public and the path is correct.

  = hint: ensure the type is public and the path is correct in your .zng file
```

**Solution:** Make sure the type is declared as `pub` and the path in your `.zng` file matches exactly.

If you need to work around layout extraction issues, you can switch to explicit layouts or `#heap_allocate`:

```
type crate::MyType {
    #layout(size = 32, align = 8);  // or #heap_allocate
}
```
