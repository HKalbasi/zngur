# Zngur

[<img alt="github" src="https://img.shields.io/badge/github-hkalbasi/zngur-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/hkalbasi/zngur)
[<img alt="crates.io" src="https://img.shields.io/crates/v/zngur.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/zngur)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-zngur-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/zngur)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/hkalbasi/zngur/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/hkalbasi/zngur/actions?query=branch%3Amain)

Zngur (/zængɑr/) is a C++/Rust interop tool. It tries to expose arbitrary Rust types, methods and functions, while preserving its
semantics and ergonomics as much as possible. Using Zngur, you can use arbitrary Rust crate in your C++ code as easily as using it in
normal Rust code, and you can write idiomatic Rusty API for your C++ library inside C++. See [the documentation](https://hkalbasi.github.io/zngur/)
for more info.

## Demo

```C++
#include <iostream>
#include <vector>

#include "./generated.h"

// Rust values are available in the `::rust` namespace from their absolute path
// in Rust
template <typename T> using Vec = rust::std::vec::Vec<T>;
template <typename T> using Option = rust::std::option::Option<T>;
template <typename T> using BoxDyn = rust::Box<rust::Dyn<T>>;

// You can implement Rust traits for your classes
template <typename T>
class VectorIterator : public rust::std::iter::Iterator<T> {
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
  auto f = BoxDyn<rust::Fn<int32_t, int32_t>>::make_box([&](int32_t x) {
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
  auto vec_as_iter = BoxDyn<rust::std::iter::Iterator<int32_t>>::make_box<
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
