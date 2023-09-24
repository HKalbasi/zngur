# Layout policy

A normal Zngur usage requires explicitly declaring data layout information (size and align) in order to storing Rust things
by value in the C++ stack. But the layout of Rust type are not stable, and can break when changing the compiler version, using
different compiler configuration (e.g. `-Z randomize-layout`) and in different targets. This can make Zngur unusable for certain
circumstances, so Zngur supports different strategies (and all of them has their own drawback) for storing Rust things by value
in the C++ stack, called "Layout policies".

In fact, you should never use this mode of Zngur (and any other form of static assertion on size and align) when you don't control
the final compiler that compiles the code, since it can break for your users in an unrecoverable state if they can't change the `zng` file
and rebuild your code. For example, you should never publish such a crate in the crates.io, if everyone does that, it makes upgrading
the compiler challenging and breaks the ecosystem.

In a normal Rust/C++ project where C++ is the boss, you usually can control the compiler version. It's perfectly fine to have multiple
versions of the Rust compiler in a single C++ build process, and Rustup will make it easy for you to pick your pinned version of the
compiler. In a binary Rust project there is also no problem, since you control the final build process, but in library Rust projects
using Zngur with explicit data layout annotations for unstable types is usually not desireable.

These are the layout policies Zngur currently supports:

## `layout(size = X, align = Y)`

This is the normal mode, which you should provide exact value of size and align for your type, which enables storing
them by value in the stack of C++. This mode has the best performance, but it has problems mentioned above.

Note that even in projects that you don't control the compiler, you can use this mode for types that have stable
size and align, such as:

- `Box<T>`, `Arc<T>`, `Rc<T>`
- `Option<&T>`
- `Vec<T>`
- `String`
- `#[repr(C)]` or `#[repr(transparent)]` structs or enums with fields with stable layout
- `[T; N]` where `T` has stable layout
- primitives

## `layout_conservative(size = X, align = Y)`

{{#include ../unimplemented_begin.md}}1{{#include ../unimplemented_end.md}}

Using this mode you can declare a size and align greater than the real ones. This will waste some amount of space, but reduces the probability
of breakage when upgrading the compiler.

## `#heap_allocate`

This policy allows owning Rust things without knowing their size at compile time. For types that uses this policy, every Rust object
construction asks Rust to heap allocate a chunk of memory with appropriate size and align, and stores data in that allocation. This
has lower performance relative to `layout` and stack allocation of objects, but doesn't need knowing size and align at the compile time.

## `#only_by_ref`

This policy deletes the constructor of the type, makes all by value usages of it a compile time error. This is a subset of `#heap_allocate`, that is,
by using `#heap_allocate` and using the object only by reference you will get the same behavior. The only point of using this instead of `#heap_allocate` is
to prevent accidental invisible heap allocations. If you don't care, use `#heap_allocate` everywhere.

## `?Sized`

Adding `?Sized` in [wellknown traits](./wellknown_traits.html) will implicitly mark the type as `#only_by_ref`.

## Example

See [`examples/rustyline`](https://github.com/HKalbasi/zngur/blob/main/examples/rustyline/main.zng) for an example that is immune to layout
breakages.
