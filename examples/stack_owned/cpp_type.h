#pragma once
#include <cstdint>
#include <iostream>
#include <memory>
#include <type_traits>
#include <zngur.h>

struct CppType {
  int x;
  int y;

  CppType() = default;
  CppType(const CppType &) = default;

  CppType(int x, int y) : x(x), y(y) {
    std::cout << "Constructed CppType " << x << " " << y << std::endl;
  }
};

struct CppConservativeType {
  int64_t a;
  int32_t b;

  CppConservativeType() = default;
  CppConservativeType(const CppConservativeType &) = default;

  CppConservativeType(int64_t a, int32_t b) : a(a), b(b) {
    std::cout << "Constructed CppConservativeType " << a << " " << b << std::endl;
  }

  ~CppConservativeType() {
    std::cout << "Destructed CppConservativeType " << a << " " << b << std::endl;
  }
};

template <>
struct rust::is_trivially_relocatable<CppConservativeType> : std::true_type {};

// `unique_ptr` is just a pointer: relocating it is a memcpy, so opt in so
// it can cross the boundary by value like any other stack C++ type.
template <>
struct rust::is_trivially_relocatable<std::unique_ptr<CppType>> : std::true_type {};

struct CppRefType {
  int v;

  CppRefType() = default;
  CppRefType(int v) : v(v) {}
};
