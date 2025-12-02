# Standalone Output Format (0.8+)

Starting with Zngur 0.8.0, the **standalone output format** is the recommended way to use Zngur, especially when using [`#layout(auto)`](./call_rust_from_cpp/auto_layout.md). This format generates a complete Rust project (a "bridge crate") that encapsulates all FFI logic.

## Overview

The standalone format solves a fundamental architectural problem: when using `#layout(auto)`, Zngur needs to inspect compiled type layouts, but traditional approaches created circular dependencies (code generation → compilation → layout extraction → code generation).

The solution is to generate a **separate bridge crate** that:

1. Contains all generated Rust and C++ FFI code
2. Depends on your main Rust crate (not the other way around)
3. Produces a single static library that includes everything

## Generated Structure

When you run `zngur gs main.zng`, Zngur creates a `zngur-bridge/` directory with this structure:

```
zngur-bridge/
├── Cargo.toml          # Bridge crate manifest (depends on your crate)
├── build.rs            # Build script (compiles C++ code)
├── src/
│   └── lib.rs          # Generated Rust FFI glue code
└── cpp/
    ├── generated.h     # Generated C++ header
    └── generated.cpp   # Generated C++ implementation (if needed)
```

## Build Requirements

### Automatic Crate Compilation

**When using `#layout(auto)`, `zngur gs` automatically builds your Rust crate if needed.** You don't need to manually run `cargo build` first - just run `zngur gs` and it handles everything.

Why? Because `#layout(auto)` works by:

1. Compiling a helper program that links against your crate
2. Using `std::mem::size_of` and `std::mem::align_of` to extract actual type layouts
3. **Embedding those values as literals into the generated Rust source code**

The key insight: **Layout extraction happens during code generation (Step 2), not during compilation (Step 3).**

The generated `bridge/src/lib.rs` contains hardcoded values like:

```rust
const _: () = assert!(std::mem::size_of::<Vec<i32>>() == 24);
```

These values must be determined before the source code is written. By the time `cargo build` runs on the bridge, the layouts are already baked into the source as integer literals.

**Why can't the bridge's build.rs extract layouts?** Because Cargo would create a circular dependency:

- Bridge's build.rs needs to compile your crate to extract layouts
- But your crate is a dependency of the bridge
- Cargo doesn't allow dependencies in build.rs to depend on the package itself

This is why `zngur gs` (a standalone tool) extracts layouts before the bridge is built.

## Typical Build Workflow

### Makefile-Based Projects

```makefile
# Step 1: Generate bridge project (auto-builds your crate if needed)
zngur-bridge/: main.zng src/lib.rs Cargo.toml
	zngur gs main.zng

# Step 2: Build bridge crate (cargo builds your crate as dependency)
zngur-bridge/target/release/libzngur_bridge.a: zngur-bridge/
	cd zngur-bridge && cargo build --release

# Step 3: Link C++ executable (only against bridge)
a.out: main.cpp zngur-bridge/target/release/libzngur_bridge.a
	$(CXX) main.cpp -L zngur-bridge/target/release -lzngur_bridge
```

### Build Script Integration

For projects using Cargo build scripts, the workflow is simpler since Cargo handles dependencies:

```rust
// build.rs
use zngur::Zngur;

fn main() {
    // Your crate is automatically built by Cargo before build.rs runs
    Zngur::from_zng_file("main.zng")
        .with_standalone_output_dir("zngur-bridge")
        .generate();

    // Then build the generated bridge
    Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir("zngur-bridge")
        .status()
        .expect("Failed to build bridge");
}
```

## Key Benefits

### 1. No Circular Dependencies

Traditional approach:

```
Your Crate → (needs) Generated Code → (needs) Compiled Crate → ❌ CIRCULAR!
```

Standalone approach:

```
Your Crate → ✅ Compiles independently
    ↓
Layout Extraction → ✅ Reads compiled crate
    ↓
Bridge Crate → ✅ Depends on your crate
    ↓
C++ → ✅ Links only against bridge
```

### 2. Single Library to Link

The bridge's static library (`libzngur_bridge.a`) includes everything:

- Your crate's code
- All dependencies
- FFI glue code
- Compiled C++ bridge code

Your C++ project only needs: `-L zngur-bridge/target/release -lzngur_bridge`

### 3. Clean Separation

The bridge crate is a completely separate Cargo project:

- Can be added to `.gitignore`
- Regenerated on demand
- No pollution of your main crate

### 4. Works with Cargo's Dependency Resolution

The bridge's `Cargo.toml` explicitly depends on your crate:

```toml
[dependencies]
my-crate = { path = "../" }
```

Cargo automatically:

- Rebuilds your crate when sources change
- Links everything correctly
- Handles feature flags and conditional compilation

## CLI Usage

### Basic Usage

```bash
zngur gs main.zng
# or
zngur generate-standalone main.zng
```

### With Options

```bash
zngur gs main.zng \
    --output-dir my-bridge \
    --cpp-namespace myproject \
    --crate-path /path/to/your/crate
```

### Available Options

- `--output-dir` / `-o`: Output directory for bridge project (default: `zngur-bridge`)
- `--cpp-namespace`: C++ namespace for generated code (default: `rust`)
- `--mangling-base`: Base string for symbol mangling (default: same as cpp-namespace)
- `--crate-path`: Path to your Rust crate (default: parent of .zng file)
- `--layout-cache-dir`: Directory for caching layout info (default: temporary dir)
- `--target`: Target triple for cross-compilation

## Comparison with Traditional Output

| Aspect               | Traditional (`zngur g`)        | Standalone (`zngur gs`) |
| -------------------- | ------------------------------ | ----------------------- |
| **Output**           | 3 separate files               | Complete Cargo project  |
| **#layout(auto)**    | ❌ Circular dependency         | ✅ Fully supported      |
| **Build complexity** | Manual coordination            | Standard Cargo workflow |
| **C++ linking**      | Link against multiple libs     | Single library          |
| **Recommended for**  | Legacy projects, custom builds | **All new projects**    |

## Migration from Traditional Format

If you have an existing project using `zngur g`, migration is straightforward:

1. **Update your build system** to run `zngur gs` instead of `zngur g`
2. **Build your crate first** (add as Make dependency or rely on Cargo)
3. **Update C++ includes** from `#include "generated.h"` to `#include "zngur-bridge/cpp/generated.h"`
4. **Update linker flags** to link only against `-lzngur_bridge`
5. **Add `zngur-bridge/` to `.gitignore`**

## When to Use Traditional Format

The traditional format (`zngur g`) is still supported for:

- **Legacy projects** with established build systems
- **When not using `#layout(auto)`** - explicit layouts work fine
- **Custom integration requirements** where you need direct control over file placement
- **Build systems that cannot handle nested Cargo projects**

However, for all new projects, especially those using `#layout(auto)`, the standalone format is strongly recommended.

## Troubleshooting

### "Could not find compiled library"

This error should rarely occur since `zngur gs` automatically builds your crate. If you see it, it typically means:

```
error: could not find compiled library
  Built crate 'my_crate' successfully, but could not find compiled library artifacts.
  = hint: ensure your Cargo.toml specifies a library crate-type
```

**Solution:** Check that your `Cargo.toml` includes a library target:

```toml
[lib]
crate-type = ["lib"]  # or ["staticlib"], ["cdylib"], etc.
```

### "The package provides no linkable target"

```
warning: The package `my_crate` provides no linkable target.
```

This warning can be ignored if your crate is a library. The bridge crate needs your crate as a dependency, not as a linkable artifact. Cargo will still build and link it correctly.

### Bridge crate won't build

If the generated bridge crate fails to build:

1. **Check that your main crate compiles**: `cargo build` in your main crate
2. **Verify the dependency path** in `zngur-bridge/Cargo.toml`
3. **Clean and rebuild**: `cd zngur-bridge && cargo clean && cargo build`

## Best Practices

1. **Add bridge to `.gitignore`**: The bridge directory is generated, not source

   ```gitignore
   zngur-bridge/
   ```

2. **Use explicit targets in Makefile**: Ensure proper build order

   ```makefile
   all: a.out

   lib: target/release/libmycrate.a

   bridge: zngur-bridge/target/release/libzngur_bridge.a
   ```

3. **Leverage Cargo for complex projects**: For projects with complex dependency graphs, integrate into `build.rs`

4. **Cache layout information**: The layout cache (default in `OUT_DIR` or temp) speeds up incremental builds

5. **Use the same target triple**: When cross-compiling, ensure your crate and the bridge use the same `--target`

## Summary

The standalone output format is the future of Zngur. It elegantly solves the circular dependency problem that made `#layout(auto)` impossible in traditional workflows, while also simplifying the build process and C++ integration.

**Key takeaway:** Compiling your Rust crate before running `zngur gs` is not a limitation - it's the essential design that makes automatic layout extraction possible and correct.
