# zngur-auto-layout

Automatic type layout extraction for zngur's `#layout(auto)` feature.

> **Note:** This feature is experimental. The intention is to make `layout(auto)` the default before 1.0.

## Usage

### Basic Usage

In your `.zng` file, use `#layout(auto)` instead of manually specifying size and alignment:

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

Zngur automatically extracts the correct size and alignment values during the build process.

### Requirements

- **Cargo must be installed** - `#layout(auto)` uses Cargo to locate and build your crate
- **rustc** - Used to compile the layout probe

If you're using a bare `rustc` workflow without Cargo, use explicit `#layout(size = X, align = Y)` directives instead.

### Build Script Configuration

Auto-layout works automatically with no special configuration. For advanced cases:

```rust
Zngur::from_zng_file(crate_dir.join("main.zng"))
    .with_layout_cache_dir(out_dir.clone())    // Cache directory (default: OUT_DIR)
    .with_crate_path(crate_dir.clone())        // Crate path (default: CARGO_MANIFEST_DIR)
    .with_target("x86_64-unknown-linux-gnu")   // Target triple (default: auto-detect)
    .generate();
```

### CLI: Dumping Layouts

Extract layouts without modifying your `.zng` file:

```bash
zngur dump-layouts main.zng
```

Output (ready to copy-paste):

```
# Extracted layouts for x86_64-apple-darwin (rustc 1.75.0)

type crate::Inventory {
    #layout(size = 32, align = 8);
}
```

For cross-compilation:

```bash
zngur dump-layouts main.zng --target x86_64-unknown-linux-gnu
```

### Troubleshooting

**"cargo is not available"**
```
error: cargo is not available
  #layout(auto) requires Cargo to extract type layouts
  = hint: install Cargo from https://rustup.rs or use explicit #layout(size = X, align = Y)
```

**"could not find compiled library"**
```
error: could not find compiled library
  = hint: run `cargo build` first to compile your crate before running zngur
```

**Workaround:** Switch to explicit layouts or `#heap_allocate`:
```
type crate::MyType {
    #layout(size = 32, align = 8);  // or #heap_allocate
}
```

---

## Architecture

### How Layout Extraction Works

When you use `#layout(auto)`, zngur:

1. **Collects types** - Parses the `.zng` file to find all types with `#layout(auto)`
2. **Generates a probe program** - Creates temporary Rust code with `#[link_section]` statics:
   ```rust
   #[used]
   #[link_section = ".zngur_0"]  // or "__DATA,__zngur0" on macOS
   static LAYOUT_0: [usize; 2] = [size_of::<MyType>(), align_of::<MyType>()];
   ```
3. **Compiles to object file** - Uses `rustc --emit=obj --extern mycrate=<path>` (no linking or execution)
4. **Parses the object file** - Reads layout values from custom sections using the `object` crate
5. **Caches results** - Stores layouts for future builds
6. **Generates code** - Uses extracted values in the C++ header

### Cross-Compilation Support

Because zngur reads layout data from compiled object files rather than executing code, `#layout(auto)` works correctly for cross-compilation. When you specify a `--target`, zngur compiles the probe for that target and extracts layouts from the resulting object file. No emulator or target hardware is required.

### Platform-Specific Section Naming

- **ELF (Linux)**: `.zngur_N`
- **Mach-O (macOS/iOS)**: `__DATA,__zngurN`
- **PE/COFF (Windows)**: `.zngur_N`

The `object` crate handles reading these sections with correct endianness and pointer size.

### Cache Invalidation

The layout cache is automatically invalidated when:

- Rust compiler version changes
- Target triple changes
- Source files are modified (detected via Cargo.lock and src/ mtime)
- Cargo features change

### Module Structure

- `extractor.rs` - Core layout extraction via rustc compilation and object file parsing
- `cache.rs` - Layout caching to avoid redundant extractions
- `types.rs` - Layout and error types

---

## Future Directions

### Cargo-Free Usage (`--rlib` flag)

The core extraction logic only requires **rustc** and a pre-compiled `.rlib` file. Cargo is currently used for convenience (locating artifacts, building if needed), but the architecture supports a cargo-free mode.

A future `--rlib` flag would allow users with alternative build systems (Bazel, Buck, plain rustc) to use `#layout(auto)`:

```bash
# User compiles their crate however they want
rustc --crate-type=rlib -o libmycrate.rlib src/lib.rs

# User runs zngur with explicit rlib path
zngur g main.zng --rlib=libmycrate.rlib --crate-name=mycrate
```

In this mode:

- No cargo check or invocation
- User provides the `.rlib` path directly
- Zngur generates the probe, compiles with `rustc --extern mycrate=<path>`, and extracts layouts

The `#[link_section]` machinery is purely internal - users don't need to modify their code or know about the extraction mechanism.
