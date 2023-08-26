# Zngur

Zngur (/zængɑr/) is a radical approach to the C++/Rust interoperability problem.

## Idea

Rust and C++ are similar languages, but with some important differences. Particularly:

- Rust is a memory safe language with strong boundary between `safe` and `unsafe`, C++
  is an unsafe language with no such difference.
- C++ has macro-like templates, which supports variadic, specialization and are checked
  at instantiation time. Rust generics on the other hand, are type checked at definition
  time using trait bounds.
- Rust move is a `memcpy` with compiler support for not destructing the moved out of variable, but C++
  move can execute arbitrary code.

In all of these differences, C++ has more freedom relative to the Rust:

- Rust considers C++ functions as unsafe, but C++ will happily call Rust code (even unsafe) as there
  is no difference between it and normal C++ code.
- Every rust generic code is valid C++ template, but not vise versa.
- C++ can simulate Rust moves very easily (by doing an actual memcpy of data, and tracking the state of destruction in
  a boolean flag) but Rust has difficulty with C++ moves. Specially, since Rust assumes that every type is
  Rust-moveable, it can never store C++ things by value, just over some indirection and `Pin`.

Considering that, Zngur tries to expose a full and detailed image of the Rust code into the C++, so that you write your Rust
as a normal Rust crate, with no `unsafe`, raw pointer, `Pin` and similar in its API, working with traits, closures and everything
that you use in your normal Rust code, and finally using it in C++ with minimal effort and maximal ease, feeling that
you are using a normal C++ library, not something in a foreign language.

## Demo

```C++
#include <iostream>
#include <vector>

#include "./generated.h"

// Rust values are available in the `::rust` namespace from their absolute path
// in Rust
template <typename T> using Vec = ::rust::std::vec::Vec<T>;
template <typename T> using Option = ::rust::std::option::Option<T>;
template <typename T> using BoxDyn = ::rust::Box<::rust::Dyn<T>>;

// You can implement Rust traits for your classes
template <typename T>
class VectorIterator : public rust::Impl<::rust::std::iter::Iterator<T>> {
  std::vector<T> vec;
  size_t pos;

public:
  VectorIterator(std::vector<T> &&v) : vec(v), pos(0) {}

  Option<T> next() override {
    if (pos >= vec.size()) {
      return Option<T>::None();
    }
    T value = vec[pos++];
    // You can construct Rust enum with fields in C++
    return Option<T>::Some(value);
  }
};

int main() {
  // You can call Rust functions that return things by value, and store that
  // value in your stack.
  auto s = Vec<int32_t>::new_();
  s.push(2);
  Vec<int32_t>::push(s, 5);
  s.push(7);
  Vec<int32_t>::push(s, 3);
  // You can call Rust functions just like normal Rust.
  std::cout << s.clone().into_iter().sum() << std::endl;
  int state = 0;
  // You can convert a C++ lambda into a `Box<dyn Fn>` and friends.
  auto f = BoxDyn<::rust::Fn<int32_t, int32_t>>::build([&](int32_t x) {
    state += x;
    std::cout << "hello " << x << " " << state << "\n";
    return x * 2;
  });
  // And pass it to Rust functions that accept closures.
  auto x = s.into_iter().map(std::move(f)).sum();
  std::cout << x << " " << state << "\n";
  std::vector<int32_t> vec{10, 20, 60};
  // You can convert a C++ type that implements `Trait` to a `Box<dyn Trait>`.
  // `make_box` is similar to the `make_unique`, it takes constructor arguments
  // and construct it inside the `Box` (instead of `unique_ptr`).
  auto vec_as_iter = BoxDyn<::rust::std::iter::Iterator<int32_t>>::make_box<
      VectorIterator<int32_t>>(std::move(vec));
  // Then use it like a normal Rust value.
  auto t = vec_as_iter.collect();
  // Some utilities are also provided. For example, `zngur_dbg` is the
  // equivalent of `dbg!` macro.
  zngur_dbg(t);
}
```

Output:

```
17
hello 2 2
hello 5 7
hello 7 14
hello 3 17
34 17
[main.cpp:61] t = [
    10,
    20,
    60,
]
```

See the `examples/simple` directory if you want to actually build and run it.

## How it compares to CXX, AutoCXX, ...?

CXX says:

> Be aware that the design of this library is intentionally restrictive and opinionated! It isn't a goal to be powerful enough to handle arbitrary signatures in either language. Instead this project is about carving out a reasonably expressive set of functionality about which we can make useful safety guarantees today and maybe extend over time.

Zngur also makes safety guarantees, but also tries to be powerful enough to handle arbitrary signatures from Rust code. And we believe
Rust types are expressive enough so that there is no need to support C++ types in Rust. For example, instead of defining a C++ opaque
type and using it in Rust by an `UniquePtr`, you can define the behavior of that type in a Rust normal trait, implement that trait
for your C++ type, and then convert that type into a `Box<dyn Trait>` and use it in Rust. Zngur benefits over CXX are:

- Zngur supports owning Rust variables on the C++ stack, saving some unnecessary allocations.
- CXX has a limited support for some types in the standard library, but Zngur supports almost everything (`Vec<T>`, `Vec<Vec<T>>`, `HashMap<T, T>`,
  `Box<[T]>`, `Arc<dyn T>`, ...) with almost full API.
- Zngur keeps the Rust side clean and normal, moving all glue code in the C++ side. But using CXX you will see the foreign types and glue codes
  in both languages.

AutoCXX is different. It also tries to be powerful enough to handle arbitrary signatures, but from C++ code. It does a nice job, but due
the fundamental difference between Rust and C++, it has several drawbacks:

- It wraps every function result that is not behind a reference in a `UniquePtr`, which has both performance and ergonomic costs.
- Every potentially mutable reference (`&mut`, `Box`, ...) should be wrapped in a `Pin`, similar to CXX.
- It has poor support for templates, though it might be temporary and an ideal AutoCXX might fix some of its problems with template.

So, an auto generated binding for a C++ library using AutoCXX is not an idiomatic and ergonomic Rust. So unless you are writing a very little
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
construct C++ objects in stack without a macro, calling their move and copy constructors using `=` operator, and ... without losing hitting any major
problem in using Rust types.

All being said, if you want to have fully auto generated bindings for both sides, nothing will prevent you from using Zngur and AutoCXX at the same
time.
