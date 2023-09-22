# Design decisions

## Keep Rust code normal and idiomatic. All glue code should live in the C++ side.

One of the most important use of a C++/Rust interop tool like Zngur is in Rust rewrite projects. In
those projects, most people are C++ experts but have little to no Rust experience. Writing
glue codes in Rust and using `UniquePtr`, `Pin` and similar make Rust code more weirder and
harder than what Rust actually is, creates a not really great first Rust experience for them.

Writing glue code in C++ also makes things considerably easier, since C++ semantics are a superset of
Rust semantics (See [idea](./zngur.md#idea)) and Rust can't express all C++ things easily.

## Be a zero cost abstraction

When Rust and C++ are used in a project, it means that performance is a requirement. So, unlike interoperability
tools between Rust and higher level languages, Zngur is not allowed to do deep copies or invisible allocations.

## Don't special case standard library types

## Keep Rust things Rusty

To minimizing the surprise. Rust decisions are usually superior to C++ ones.

### `Result<T, E>` is not automatically converted to exception

### Copy constructors are deleted, manual `.clone()` should be used

### `RustType r;` has the same semantics as `let r: RustType;`

### Rust functions returning `()` do the same in C++

This one has very little practical benefits, and might be revisited in future.
