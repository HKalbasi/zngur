# Automatic Layout Resolution

Starting with Zngur 0.8.0, you can automatically determine type size and alignment at build time using `#layout(auto)` instead of manually specifying `#layout(size = X, align = Y)`.

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

## Build Script Configuration

Auto-layout works automatically with no special configuration needed in your `build.rs`:

```rust
use zngur::Zngur;

fn main() {
    let crate_dir = build::cargo_manifest_dir();
    let out_dir = build::out_dir();

    Zngur::from_zng_file(crate_dir.join("main.zng"))
        .with_cpp_file(out_dir.join("generated.cpp"))
        .with_h_file(out_dir.join("generated.h"))
        .with_rs_file(out_dir.join("generated.rs"))
        .generate();
}
```

If there are no types with `#layout(auto)` in your `.zng` file, the auto-layout system simply skips all processing with no overhead.

### Advanced Configuration

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

This ensures the cache never becomes stale.

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

## When to Use Auto Layout

For most use cases where you control the compiler version, **`#layout(auto)` is the recommended default**. Here's the full decision tree:

### Quick Decision Guide

**For types with unstable layouts** (most custom types, tuples, etc.):

- ✅ Use `#layout(auto)` - convenient and always correct
- ⚠️ Never use explicit `#layout(size = X, align = Y)` in published libraries (see [Layout Policy](./layout_policy.md))
- ✅ Use `#heap_allocate` if you need maximum portability across compiler versions

**For types with stable layouts** (`Vec<T>`, `String`, `Box<T>`, `#[repr(C)]` types, primitives):

- ✅ Use `#layout(auto)` - convenient and verifies your assumptions
- ✅ Use explicit `#layout(size = X, align = Y)` - slightly faster builds, no rustc dependency
- Both work fine; auto is safer because it catches mistakes

### Detailed Considerations

**Use `#layout(auto)` when:**

- You want convenience (no manual lookup)
- You control the compiler version (most projects)
- You want automatic verification that values are correct
- The type's layout might change between compiler versions

**Use explicit `#layout(size = X, align = Y)` only when:**

- The type has a **guaranteed stable** layout
- You want to avoid rustc as a build dependency
- You're okay with manual maintenance
- Examples: `Vec<T>` (24 bytes), `String` (24 bytes), `Box<T>` (8 bytes), primitives

**Use `#heap_allocate` when:**

- You need maximum portability (works with any compiler version)
- You don't control the final build environment
- Heap allocation overhead is acceptable
- You don't want to maintain layout information at all

## Limitations

- Requires the crate to be built before layout extraction
- Types must be public or accessible from the crate root
- Cross-compilation requires the target's stdlib to be installed
- Adds a small amount of build time (typically < 1 second, cached)

## Example

See [`examples/tutorial`](https://github.com/HKalbasi/zngur/blob/main/examples/tutorial) for a complete example using auto-layout.

## Troubleshooting

Auto-layout errors include helpful hints to guide you toward a solution:

```
error: could not find compiled library
  Could not find compiled library for crate 'my_crate' in ./target/debug/

  = hint: run `cargo build` first to compile your crate before running zngur
```

**Solution:** Run `cargo build` to compile your crate first.

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
