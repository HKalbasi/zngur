# Tutorial

A Zngur project consists of 3 things:

- An IDL (interface definition language) file named `main.zng`
- A Rust crate (that can be everything, binary, rlib, static-lib, cdy-lib, ...)
- A C++ project.

For start, generate a new `staticlib` crate using `cargo init` and appending this to the `Cargo.toml`:

```Toml
[lib]
crate-type = ["staticlib"]
```

And create an empty `main.cpp` and `main.zng` file. Your directory tree should look like this:

```
├── Cargo.toml
├── main.cpp
├── main.zng
└── src
    └── lib.rs
```

## Basic structure of `main.zng`
