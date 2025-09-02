# Design decisions

## Keep Rust code normal and idiomatic. All glue code should live on the C++ side.

One of the most important uses of a C++/Rust interop tool like Zngur is in Rust rewrite projects.
In those projects, most people are C++ experts but have little to no Rust experience.
Writing glue code in Rust and using `UniquePtr`, `Pin` and similar constructs make Rust code weirder
and harder than what Rust actually is, creating a not really great first Rust experience for them.

Writing glue code in C++ also makes things considerably easier,
since C++ semantics are a superset of Rust semantics (See [idea](./zngur.md#idea))
and Rust can't express all C++ things easily.

### Keep `main.zng` in a separate file

CXX-style embedding of the IDL in a Rust proc macro confuses Rust newcomers, so Zngur avoids it.

## Be a zero cost abstraction

When Rust and C++ are used in a project, it means that performance is a requirement. So, unlike interoperability
tools between Rust and higher level languages, Zngur is not allowed to do deep copies or invisible allocations.

## Be build system agnostic

C/C++ build systems are complex, each in a different way.
To support all of them, Zngur doesn't integrate with any of them.
Any build system that can do the following process is able to build a Zngur project:

- Running `zngur g main.zng`
- Building the Rust project (e.g. by running the `cargo build`)
- Build the C++ `generated.cpp` together with the rest of codes
- Link all together

## Keep Rust things Rusty

- To minimize surprise.
- Rust decisions are usually superior to C++ ones.

### `Result<T, E>` is not automatically converted to exception

`Result<T, E>` has some benefits over exception-based error handling.
For example, the unhappy case cannot be forgotten and must be handled.
Due to these benefits, a similar `std::expected<T, E>` was added to C++23.
In order to not lose this Rust benefit, `Result<T, E>` is not converted to a C++ exception.

Panics, which are implemented by stack unwinding similar to C++ exceptions,
are converted to a C++ exception with the [`#convert_panic_to_exception`](./call_rust_from_cpp/panic_and_exceptions.md) flag.
So if you quickly want an exception out of a `Result<T, E>`, you can use `.unwrap()`.

### Copy constructors are deleted, manual `.clone()` should be used

Implicit copy constructors are a source of accidental performance cost,
and complicate the control flow of program.
Rust doesn't support them and uses explicit `.clone()` calls for that purpose, and Zngur follows Rust.

Note that for `Copy` types, where the move operation is not destructive,
Zngur doesn't delete the copy constructor.

### `RustType r;` has the same semantics as `let r: RustType;`

Normally in C++ the default constructor creates a basic initialized object,
and you can use it immediately after that.
For example, this code is valid:

```C++
std::vector<int32_t> v;
v.push_back(2);
```

But in Zngur, default constructor always exists and creates an uninitialized object, so this code is invalid:

```C++
rust::std::vec::Vec<int32_t> v;
v.push(2);
```

An alternative would be running `Default::default()` in the default constructor.
This behavior is selected over that,
because it can enable somethings that are not possible without it with the same performance,
such as conditional initialization:

```C++
Vec<int32_t> v;
if (reserve_capacity) {
    v = Vec<int32_t>::with_capacity(1000);
} else {
    v = Vec<int32_t>::new_();
}
```

If `Vec<int32_t> v` used the default constructor,
it would be a waste call to itself and a wasted call to the drop code executed immediately after that.
Rust also support this, but checks the initialization before usage,
which Zngur can't check in the compile time, but will check in the run time by default.

### Rust functions returning `()` return `rust::Unit` in C++ instead of `void`

This one has very little practical benefits, and might be revisited in future.
