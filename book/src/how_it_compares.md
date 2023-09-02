# How it compares to other tools?

Zngur is not the only tool in the Rust/C++ interoperability space. Its pros and cons relative to other popular tools are described
on this page.

## CXX

CXX says:

> Be aware that the design of this library is intentionally restrictive and opinionated! It isn't a goal to be powerful enough to handle arbitrary signatures in either language. Instead, this project is about carving out a reasonably expressive set of functionality about which we can make useful safety guarantees today and maybe extend over time.

Zngur also makes safety guarantees but also tries to be powerful enough to handle arbitrary signatures from Rust code. And we believe
Rust types are expressive enough so that there is no need to support C++ types in Rust. For example, instead of defining a C++ opaque
type and using it in Rust by an `UniquePtr`, you can define the behavior of that type in a Rust normal trait, implement that trait
for your C++ type, and then convert that type into a `Box<dyn Trait>` and use it in Rust. Zngur benefits over CXX are:

- Zngur supports owning Rust variables on the C++ stack, saving some unnecessary allocations.
- CXX has limited support for some types in the standard library, but Zngur supports almost everything (`Vec<T>`, `Vec<Vec<T>>`, `HashMap<T, T>`,
  `Box<[T]>`, `Arc<dyn T>`, ...) with almost full API.
- Zngur keeps the Rust side clean and normal, moving all glue code in the C++ side. But using CXX you will see the foreign types and glue codes
  in both languages.

### Migrating from CXX to Zngur

- Create a `main.zng` file.
- Add everything in the `extern "Rust"` block into the `main.zng`. Zngur should support everything supported by CXX.
- For every opaque type in the `extern "C++"` and its methods, write an equivalent trait.
- For builtin C++ types that CXX supports:
  - `CxxString`:
    - Convert it to a Rust `&CStr`, `&[u8]`, or `&str` if ownership is not required.
    - Copy it into a Rust `CString`, `Vec<u8>`, or `String` if the performance cost is acceptable.
    - Write a trait for functionalities you need from it and convert the string to `Box<dyn Trait>`
  - `CxxVector<uintX_t>`:
    - Similar to `CxxString`
  - `CxxVector<opaque_type>`:
    - Copy it into a Rust `Vec<Box<dyn Trait>>` if the performance cost is acceptable.
    - Write a trait and wrap it like a separate opaque type.

## AutoCXX

AutoCXX is different. It also tries to be powerful enough to handle arbitrary signatures but from C++ code. It does a nice job, but due
the fundamental difference between Rust and C++, it has several drawbacks:

- It wraps every function result that is not behind a reference in a `UniquePtr`, which has both performance and ergonomic costs.
- Every potentially mutable reference (`&mut`, `Box`, ...) should be wrapped in a `Pin`, similar to CXX.
- It has poor support for templates, though it might be temporary and an ideal AutoCXX might fix some of its problems with template.

So, an auto-generated binding for a C++ library using AutoCXX is not an idiomatic and ergonomic Rust. So unless you are writing a very little
amount of Rust code, or can tolerate `UniquePtr` and `Pin` to be propagated in your Rust code, you will need to write a Rustic wrapper for
your C++ classes. This is even recommended in the AutoCXX docs:

> The C++ API may have documented usage invariants. Your ideal is to encode as many as possible of those into compile-time checks in Rust.
> Some options to consider:
> Wrap the bindings in a new type wrapper which enforces compile-time variants in its APIs; for example, taking a mutable reference to enforce exclusive access.

Using Zngur, you can write that wrapper in C++! Zngur supports implementing Rust traits for C++ types and casting them to
the `Box<dyn Trait>`, implementing inherent methods on Rust types, implementing Rust traits for Rust types, and calling bare functions
in C++ that operate on Rust types. Using those, you should be able to create idiomatic Rust wrappers for your C++ library.

Idiomatic C++ and idiomatic Rust are like night and day, so a glue code between them must exist. The difference between Zngur and AutoCXX is that
using AutoCXX that glue code will live in Rust and using Zngur it will live in C++. By writing the glue code in C++, you will become able to
construct C++ objects in the stack without a macro, calling their move and copy constructors using `=` operator, and ... without hitting any major
problems in using Rust types.

All being said, if you want to have fully auto-generated bindings for both sides, nothing will prevent you from using Zngur and AutoCXX at the same
time.
