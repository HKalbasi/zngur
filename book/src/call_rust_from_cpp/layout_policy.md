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

## `layout_conservative(size = X, align = Y)`

{{#include ../unimplemented_begin.md}}1{{#include ../unimplemented_end.md}}

## `#heap_allocate`

## `#only_by_ref`

This policy deletes the constructor of the type, makes all by value usages of it a compile time error. This is a subset of `#heap_allocate`, that is,
by using `#heap_allocate` and using the object only by reference you will get the same behavior. The only point of using this instead of `#heap_allocate` is
to prevent accidental invisible heap allocations. If you don't care, use `#heap_allocate` everywhere.

## `?Sized`

Adding `?Sized` in [wellknown traits](./wellknown_traits.html) will implicitly mark the type as `#only_by_ref`.
