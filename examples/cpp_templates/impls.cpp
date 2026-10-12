#include "cpp_templates.h"

#include <iostream>

#include "generated.h"

namespace rust {

// Stack templates name the real instantiation directly, including nested
// ones like `shared_ptr<vector<int32_t>>`.
::std::vector<int32_t> Impl<::std::vector<int32_t>>::make_numbers() {
  return ::std::vector<int32_t>{1, 2, 3};
}

int32_t Impl<::std::vector<int32_t>>::sum(
    rust::Ref<::std::vector<int32_t>> v) {
  int32_t total = 0;
  for (int32_t x : v.cpp()) {
    total += x;
  }
  return total;
}

rust::Unit Impl<::std::vector<int32_t>>::push(
    rust::RefMut<::std::vector<int32_t>> v,
    int32_t x) {
  v.cpp().push_back(x);
  return {};
}

::std::shared_ptr<::std::vector<int32_t>>
Impl<::std::shared_ptr<::std::vector<int32_t>>>::make_shared_numbers() {
  return ::std::make_shared<::std::vector<int32_t>>(
      ::std::vector<int32_t>{10, 20});
}

int32_t Impl<::std::shared_ptr<::std::vector<int32_t>>>::sum(
    rust::Ref<::std::shared_ptr<::std::vector<int32_t>>> p) {
  int32_t total = 0;
  for (int32_t x : *p.cpp()) {
    total += x;
  }
  return total;
}

rust::Unit Impl<::std::shared_ptr<::std::vector<int32_t>>>::push(
    rust::RefMut<::std::shared_ptr<::std::vector<int32_t>>> p,
    int32_t x) {
  p.cpp()->push_back(x);
  return {};
}

// Heap templates are boxed from a by-value return; the heap box carries
// the destructor, so Rust just holds the pointer.
::Holder<::CppType> Impl<::Holder<::CppType>>::make_holder(
    int32_t x,
    int32_t y) {
  return ::Holder<::CppType>(::CppType(x, y));
}

int32_t Impl<::Holder<::CppType>>::first(
    rust::Ref<::Holder<::CppType>> h) {
  return h.cpp().value.x;
}

rust::Unit Impl<::Holder<::CppType>>::set_first(
    rust::RefMut<::Holder<::CppType>> h,
    int32_t x) {
  h.cpp().value.x = x;
  return {};
}

// Borrowed templates never cross ownership: a raw pointer in, `.cpp()`
// to read back out.
::Holder<int32_t> const *Impl<::Holder<int32_t>>::default_holder() {
  static ::Holder<int32_t> instance{42};
  return &instance;
}

int32_t Impl<::Holder<int32_t>>::value(
    rust::Ref<::Holder<int32_t>> h) {
  return h.cpp().value;
}

// Mixed: a C++ template holding a Rust value. The mirror is built with
// its declared constructor and moved with the mirror's move constructor,
// which steals the drop flag; reading back moves the value out (the
// holder is left moved-from, like any moved-from C++ object).
::Holder<rust::crate::RustPoint>
Impl<::Holder<rust::crate::RustPoint>>::make_rust_holder(
    int32_t x,
    int32_t y) {
  return ::Holder<rust::crate::RustPoint>(rust::crate::RustPoint(x, y));
}

rust::crate::RustPoint
Impl<::Holder<rust::crate::RustPoint>>::take_inner(
    rust::RefMut<::Holder<rust::crate::RustPoint>> h) {
  return ::std::move(h.cpp().value);
}

// Mixed the other way: a C++ vector owning Rust boxes. Boxes cross by
// value with the mirror protocol (memcpy plus drop flag); the vector
// moves them with the mirror's move constructor, which steals the flag,
// so every allocation is freed exactly once.
::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>
Impl<::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>>::
    make_boxed_points() {
  return {};
}

int32_t
Impl<::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>>::len(
    rust::Ref<::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>>
        v) {
  return v.cpp().size();
}

rust::Unit
Impl<::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>>::push_box(
    rust::RefMut<::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>>
        v,
    rust::std::boxed::Box<rust::crate::RustPoint> b) {
  v.cpp().push_back(::std::move(b));
  return {};
}

rust::std::boxed::Box<rust::crate::RustPoint>
Impl<::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>>::take_first(
    rust::RefMut<::std::vector<rust::std::boxed::Box<rust::crate::RustPoint>>>
        v) {
  return ::std::move(v.cpp()[0]);
}

} // namespace rust
