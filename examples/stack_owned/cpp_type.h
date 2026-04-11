#pragma once
#include <iostream>
#include <type_traits>
#include <zngur.h>

struct CppType {
  int x;
  int y;

  CppType(int x, int y) : x(x), y(y) {
    std::cout << "Constructed CppType " << x << " " << y << std::endl;
  }
  ~CppType() {
    std::cout << "Destructed CppType " << x << " " << y << std::endl;
  }
};

namespace rust {
template <> struct is_trivially_relocatable<CppType> : std::true_type {};
} // namespace rust
