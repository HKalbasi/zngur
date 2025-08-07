# Wasm32 Example

This example builds a sample application for wasmtime to test zngur's support for basic WASM applications.

## To build and run

```
$ make run
```

This automatically installs all dependencies (wasmtime, WASI SDK, Rust targets) and runs the example.

## Alternative targets

```
$ make            # Same as 'make run'
$ make main.wasm  # Build without running
$ make a.out      # Create executable wrapper script
$ make clean      # Clean all build artifacts
```
