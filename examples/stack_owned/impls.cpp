#include "cpp_type.h"

#include <iostream>

#include "generated.h"

namespace rust {
namespace exported_functions {

rust::crate::MyCppWrapper create_cpp_type(int32_t x, int32_t y) {
  rust::crate::MyCppWrapper w;
  new (&w.cpp()) CppType(x, y);
  ::rust::__zngur_internal_assume_init(w);
  return w;
}

rust::Unit print_cpp_type(rust::Ref<rust::crate::MyCppWrapper> c) {
  const CppType &cpp = c.cpp();
  std::cout << "CppType " << cpp.x << " " << cpp.y << std::endl;
  return rust::Unit{};
}

} // namespace exported_functions
} // namespace rust
