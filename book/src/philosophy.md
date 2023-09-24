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

- To minimizing the surprise.
- Rust decisions are usually superior to C++ ones.

### `Result<T, E>` is not automatically converted to exception

### Copy constructors are deleted, manual `.clone()` should be used

Implicit copy constructors are a source of accidental performance cost, and complicate the control flow of program. Rust doesn't support
them and uses explicit `.clone()` calls for that propose, and Zngur follows Rust.

Note that for `Copy` types, where the move operation is not destructive, Zngur doesn't delete the copy constructor.

### `RustType r;` has the same semantics as `let r: RustType;`

Normally in C++ the default constructor creates a basic initialized object, and you can use it immediately after that. For example, this
code is valid:

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
This behavior is selected over that, because it can enable somethings that are not possible without it with the same performance, such as
conditional initialization:

```C++
Vec<int32_t> v;
if (reserve_capacity) {
    v = Vec<int32_t>::with_capacity(1000);
} else {
    v = Vec<int32_t>::new_();
}
```

If `Vec<int32_t> v` used the default constructor, it would be a waste call to itself and a wasted call to the drop code
executed immediately after that. Rust also support this, but checks the initialization before usage, which Zngur can't check.

### Rust functions returning `()` return `rust::Unit` in C++ instead of `void`

This one has very little practical benefits, and might be revisited in future.
