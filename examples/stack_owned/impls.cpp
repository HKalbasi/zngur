#include "cpp_type.h"

#include <iostream>

#include "generated.h"

namespace rust {

rust::crate::MyCppWrapper
Impl<rust::crate::MyCppWrapper>::custom_constructor(int32_t x, int32_t y) {
  return rust::crate::MyCppWrapper::build(x, y);
}

rust::Unit
Impl<rust::crate::MyCppWrapper>::print(rust::Ref<rust::crate::MyCppWrapper> c) {
  const CppType &cpp = c.cpp();
  std::cout << "CppType " << cpp.x << " " << cpp.y << std::endl;
  return {};
}

} // namespace rust
