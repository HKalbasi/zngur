# How Zngur compares to other tools

Zngur is not the only tool in the Rust/C++ interoperability space.
Its pros and cons relative to other popular tools are described on this page.

## CXX

CXX says:

> Be aware that the design of this library is intentionally restrictive and opinionated!
> It isn't a goal to be powerful enough to handle arbitrary signatures in either language.
> Instead, this project is about carving out a reasonably expressive set of functionality
> about which we can make useful safety guarantees today and maybe extend over time.

Zngur also makes safety guarantees but also tries to be powerful enough to handle arbitrary signatures from Rust code,
including generics, `impl` and `dyn` traits, and owned objects. So with Zngur you can use almost any Rust library in C++,
but with CXX you need to write some manual glue code (potentially with runtime costs) to avoid inaccessible types and signatures.

For using C++ in Rust, CXX and Zngur both provide opaque C++ types behind indirection,
but in CXX that indirection is explicit in the Rust side, usually involving `Pin` and `cxx::UniquePtr`,
while Zngur forces you to hide C++ types behind some idiomatic and safe Rust api,
which is a glue code you need to write in C++.
For example, instead of defining a C++ opaque type and using it in Rust via a `UniquePtr`,
you can define the behavior of that type in a normal Rust trait,
implement that trait for your C++ type,
and then convert that type into a `Box<dyn Trait>` and use it in Rust,
or define some Rust struct which holds this C++ object using `ZngurCppOpaqueOwned` and write methods
and implement traits for that struct in C++ (which has access to the underlying C++ object).
So Zngur and CXX have almost equivalent expressing power in this side, but with different styles.

Zngur's benefits over CXX are:

- Zngur supports owning Rust variables on the C++ stack, saving some unnecessary allocations.
- CXX has limited support for some types in the standard library,
  but Zngur supports almost everything (`Vec<T>`, `Vec<Vec<T>>`, `HashMap<T, T>`, `Box<[T]>`, `Arc<dyn T>`, ...)
  with almost full API.
- Zngur keeps the Rust side clean and normal, moving all glue code to the C++ side.
  But using CXX you will see the foreign types and glue code in both languages.

### Migrating from CXX to Zngur

- Create a `main.zng` file.
- Add everything in the `extern "Rust"` block into the `main.zng`.
  Zngur should support everything supported by CXX.
- For every opaque type in the `extern "C++"` and its methods, write an equivalent trait or opaque type.
- For builtin C++ types that CXX supports:
  - `CxxString`:
    - Convert it to a Rust `&CStr`, `&[u8]`, or `&str` if ownership is not required.
    - Copy it into a Rust `CString`, `Vec<u8>`, or `String` if the performance cost is acceptable.
    - Write a trait for functionalities you need from it and convert the string to `Box<dyn Trait>`.
    - Write an opaque type `CxxString` and implement the functionalities you need in `impl`.
  - `CxxVector<uintX_t>`:
    - Similar to `CxxString`
  - `CxxVector<opaque_type>`:
    - Copy it into a Rust `Vec<Box<dyn Trait>>` if the performance cost is acceptable.
    - Write two opaque types `VectorFoo` and `Foo`,
      where `VecFoo` has a `get(usize) -> &Foo` method.

## AutoCXX

AutoCXX is different.
It also tries to be powerful enough to handle arbitrary signatures but from C++ code.
It does a nice job, but due the fundamental difference between Rust and C++, it has several drawbacks:

- It wraps every function result that is not behind a reference in a `UniquePtr`,
  which has both performance and ergonomic costs.
- Every potentially mutable reference (`&mut`, `Box`, ...) should be wrapped in a `Pin`, similar to CXX.
- It has poor support for templates,
  though it might be temporary and an ideal AutoCXX might fix some of its problems with template.

So, an auto-generated binding for a C++ library using AutoCXX is not an idiomatic and ergonomic Rust.
So unless you are writing a very little amount of Rust code,
or can tolerate `UniquePtr` and `Pin` to be propagated in your Rust code,
you will need to write a Rustic wrapper for your C++ classes.
This is even recommended in the AutoCXX docs:

> The C++ API may have documented usage invariants. Your ideal is to encode as many as possible of those into compile-time checks in Rust.
> Some options to consider:
> Wrap the bindings in a new type wrapper which enforces compile-time variants in its APIs;
> for example, taking a mutable reference to enforce exclusive access.

Using Zngur, you can write that wrapper in C++!
Zngur supports implementing Rust traits for C++ types and casting them to the `Box<dyn Trait>`,
implementing inherent methods on Rust types,
implementing Rust traits for Rust types,
and calling bare functions in C++ that operate on Rust types.
Using those, you should be able to create idiomatic Rust wrappers for your C++ library.

Idiomatic C++ and idiomatic Rust are like night and day, so a glue code between them must exist.
The difference between Zngur and AutoCXX is that using AutoCXX that glue code will live in Rust
and using Zngur it will live in C++.
By writing the glue code in C++, you will become able to construct C++ objects in the stack without a macro,
calling their move and copy constructors using `=` operator,
and ... without hitting any major problems in using Rust types.

### Safety of AutoCXX

Idiomatic Rust libraries encapsulate unsafe, and expose safe functions for user code.
Safety is another reason that a Rust wrapper should exist between Rust most of their API surface.
Safe functions in Rust are designed to be memory safe even in presence of adversary safe code.
(Adversary safe code can exploit soundness bugs in the compiler, but that's the north star Rust follows,
and it is not that far from it) Almost no C++ API is designed with that level of safety in mind.
By writing a Rusty wrapper for C++ APIs, we can think about safety and design a safe (in Rust terms) API.

AutoCXX has a pretty hand-wavy approach to safety. It gives you three options:

- Make all functions safe, which is almost always wrong.
- Make all functions unsafe, which leads to spamming unsafe keyword everywhere, making unsafe lose
  its meaning.
- Make all functions which uses C++ references as unsafe, and all other functions safe. The fact that
  this option exists shows that declaring all functions as safe isn't working very well.

From the AutoCXX docs:

> Irrespective, C++ code is of course unsafe.
> It’s worth noting that use of C++ can cause unexpected unsafety at a distance in faraway Rust code.
> As with any use of the unsafe keyword in Rust,
> you the human are declaring that you’ve analyzed all possible ways that the code can be used
> and you are guaranteeing to the compiler that no badness can occur. Good luck.

### Using Zngur and AutoCXX together

All being said, if you want to have fully auto-generated bindings for both sides,
nothing will prevent you from using Zngur and AutoCXX at the same time.

## Crubit

Crubit is similar to AutoCXX, but it heavily integrates with rust and C++ compilers so it can handle owned
types of other sides without manual annotations.
The problems of AutoCXX about idiomatic and safe APIs applies to Crubit as well.

Crubit requires heavy control over the Rust and C++ build system,
but Zngur can work with almost any C++ compiler and any build system, as it only generates some header files.

An auto generator for zng files is planned, which can bridge the gap between Zngur and Crubit in ease of use,
but currently it does not exist.

At the time of writing this, Crubit is limited to Google build system and is not ready for general use.
It also lacks some expressive power relative to Zngur, for example it can't handle generics and `dyn` traits,
but it has plans to change this.
