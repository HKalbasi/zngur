#include "cpp_type.h"

#include <iostream>

#include "generated.h"

namespace rust {

// Method bodies for the real C++ types. There is no rust:: mirror wrapper:
// `Impl` is specialized directly on ::CppType/::CppConservativeType.
::CppType Impl<::CppType>::new_(int32_t x, int32_t y) {
  return ::CppType(x, y);
}
rust::Unit Impl<::CppType>::print(rust::Ref<::CppType> c) {
  const ::CppType &cpp = c.cpp();
  std::cout << "CppType " << cpp.x << " " << cpp.y << std::endl;
  return {};
}

::CppConservativeType Impl<::CppConservativeType>::new_(int64_t a,
                                                        int32_t b) {
  return ::CppConservativeType(a, b);
}
rust::Unit
 Impl<::CppConservativeType>::print(
    rust::Ref<::CppConservativeType> c) {
  const ::CppConservativeType &cpp = c.cpp();
  std::cout << "CppConservativeType " << cpp.a << " " << cpp.b << std::endl;
  return {};
}

// Reference-only types also name the real C++ type directly: raw pointers
// spell as plain C++ pointers and `Ref` exposes the object via `.cpp()`.
::CppRefType const *Impl<::CppRefType>::get_global() {
  static ::CppRefType instance{7};
  return &instance;
}

rust::Unit Impl<::CppRefType>::print(rust::Ref<::CppRefType> c) {
  const ::CppRefType &cpp = c.cpp();
  std::cout << "CppRefType " << cpp.v << std::endl;
  return {};
}

// Template instantiations name the real C++ type directly, so the method
// bodies are ordinary C++ (here, ordinary `unique_ptr` usage).
::std::unique_ptr<::CppType> Impl<::std::unique_ptr<::CppType>>::make_unique(
    int32_t x,
    int32_t y) {
  return ::std::unique_ptr<::CppType>(new ::CppType(x, y));
}

rust::Unit Impl<::std::unique_ptr<::CppType>>::print(
    rust::Ref<::std::unique_ptr<::CppType>> p) {
  const ::std::unique_ptr<::CppType> &cpp = p.cpp();
  std::cout << "UniqueCppType " << cpp->x << " " << cpp->y << std::endl;
  return {};
}

} // namespace rust
