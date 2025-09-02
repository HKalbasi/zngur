# Calling C++ from Rust

Until this point, all Zngur features discussed were about using Rust code inside C++,
but no C++/Rust interop tool is complete without supporting the C++ to Rust direction.
In fact, this direction is arguably more important since in a C++/Rust project most of the existing code is in C++
and Rust code needs a way to use it in order to be useful.
So Zngur also supports this direction.

Zngur's general idea is that Rust semantics is a subset of C++ semantics,
so we should use Rust things in C++ and avoid bringing C++ things into Rust (See [Design decisions](../philosophy.md)).
So, even in the C++ to Rust direction, Zngur operates only on Rust types.
For example, Zngur allows you to call a C++ function that takes Rust types as inputs in Rust,
but you can't call a function that takes a C++ object.
Or you can write an `impl` block for a Rust type in C++ and call those methods in Rust,
but you can't call C++ methods of a C++ object in Rust.

So you can't call your C++ code directly in Rust, and you need to write a Rusty wrapper for your C++ code.
This is often unavoidable even if the interop tool supports calling C++ directly,
if you have a meaningful amount of Rust code, since C++ values cannot be owned in Rust,
and Rust is more expressive in APIs and function signatures (e.g. it has lifetimes) so such a wrapper is required anyway.
Zngur tries to enable writing a Rusty wrapper for your C++ code as idiomatic as normal Rust code,
and all features in this section try to help achieve this goal.
