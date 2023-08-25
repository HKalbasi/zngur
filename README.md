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
