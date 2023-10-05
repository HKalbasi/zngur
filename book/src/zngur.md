# Zngur

Zngur (/zængɑr/) is a C++/Rust interop tool. It tries to expose arbitrary Rust types, methods, and functions while preserving its
semantics and ergonomics as much as possible. Using Zngur, you can use arbitrary Rust crate in your C++ code as easily as using it in
normal Rust code, and you can write idiomatic Rusty API for your C++ library inside C++.

## Idea

Rust and C++ are similar languages but with some important differences. Particularly:

- Rust is a memory-safe language with a strong boundary between `safe` and `unsafe` declared using the `unsafe` keyword, C++
  is an unsafe language with no such difference and keyword.
- C++ has macro-like templates, which support variadic, specialization and are checked
  at instantiation time. Rust generics, on the other hand, are type-checked at definition
  time using trait bounds.
- Rust move is a `memcpy` with compiler support for not destructing the moved out of variable, but C++
  move can execute arbitrary code.

In all of these differences, C++ has more freedom relative to the Rust:

- Rust considers C++ functions as unsafe, but C++ will happily call Rust code (even unsafe) as there
  is no difference between it and normal C++ code.
- Every rust generic code is a valid C++ template, but not vice versa.
- C++ can simulate Rust moves very easily (by doing an actual `memcpy` of data, and tracking the state of destruction in
  a boolean flag) but Rust has difficulty with C++ moves. Specially, since Rust assumes that every type is
  Rust-moveable, it can never store C++ objects by value, just over some indirection and `Pin`.

So, Zngur allows you to use arbitrary Rust types in C++, store them by value in the C++ stack, and call arbitrary Rust methods and functions
on them. But it doesn't bridge any C++ type into Rust, since it is not possible with the same ergonomic. Instead, Zngur allows you to
write a rusty wrapper for your C++ library. It allows you to implement Rust traits for C++ types and cast them to
the `Box<dyn Trait>`, implement inherent methods on Rust types, implement Rust traits for Rust types, and expose bare functions
from C++ that operate on Rust types.

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
class VectorIterator : public ::rust::std::iter::Iterator<T> {
  std::vector<T> vec;
  size_t pos;

public:
  VectorIterator(std::vector<T> &&v) : vec(v), pos(0) {}
  ~VectorIterator() {
    std::cout << "vector iterator has been destructed" << std::endl;
  }

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
  // You can catch Rust panics as C++ exceptions
  try {
    std::cout << "s[2] = " << *s.get(2).unwrap() << std::endl;
    std::cout << "s[4] = " << *s.get(4).unwrap() << std::endl;
  } catch (rust::Panic e) {
    std::cout << "Rust panic happened" << std::endl;
  }
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
s[2] = 7
thread '<unnamed>' panicked at 'called `Option::unwrap()` on a `None` value', examples/simple/src/generated.rs:186:39
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
s[4] = Rust panic happened
hello 2 2
hello 5 7
hello 7 14
hello 3 17
34 17
vector iterator has been destructed
[main.cpp:71] t = [
    10,
    20,
    60,
]
```

See the [`examples/simple`](https://github.com/HKalbasi/zngur/blob/main/examples/simple) if you want to build and run it.
