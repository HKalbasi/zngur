#pragma once
#include <cstdint>
#include <memory>
#include <type_traits>
#include <utility>
#include <vector>
#include <zngur.h>

struct CppType {
  int x;
  int y;

  CppType() = default;
  CppType(const CppType &) = default;

  CppType(int x, int y) : x(x), y(y) {}
};

template <typename T> struct Holder {
  T value;

  Holder(T v) : value(std::move(v)) {}
};

// Relocating a vector or a shared pointer is a memcpy (their heap state
// is untouched), so opt in to crossing the boundary by value.
template <>
struct rust::is_trivially_relocatable<std::vector<int32_t>>
    : std::true_type {};
template <>
struct rust::is_trivially_relocatable<std::shared_ptr<std::vector<int32_t>>>
    : std::true_type {};
// A `Holder` of plain data is plain data itself: moving it is a memcpy
// and the destructor is trivial, so the same holds with a Rust mirror
// inside (sound only while the Rust type stays `Drop`-free). The mirrors
// are only forward-declared here (they are defined by the generated
// header below); naming them is enough for the specializations, which
// live in `namespace rust` next to the primary template.
namespace rust {
namespace crate {
struct RustPoint;
} // namespace crate
namespace std {
namespace boxed {
template <typename... T> struct Box;
} // namespace boxed
} // namespace std
template <>
struct is_trivially_relocatable<::Holder<crate::RustPoint>>
    : ::std::true_type {};
// A vector of Rust boxes is just three pointers; the box mirrors ride
// along via their flag-stealing move constructor.
template <>
struct is_trivially_relocatable<
    ::std::vector<std::boxed::Box<crate::RustPoint>>>
    : ::std::true_type {};
} // namespace rust
