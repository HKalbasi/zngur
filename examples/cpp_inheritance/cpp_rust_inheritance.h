#pragma once

#include "zngur.h"

namespace rust {
template <typename CppBase, typename RustDerived> class RustInherit {
public:
  static RawMut<RustDerived> get_rust(CppBase *base);
  static CppBase *get_base(RawMut<RustDerived> derived);

private:
  static void *AlignedTo(void *unaligned_addr, size_t alignment);
  static size_t DerivedAlign();
  static size_t FullAlign();
};

template <typename T> RawMut<T> as_rust_ptr_mut(T *t) {
  return *reinterpret_cast<RawMut<T> *>(&t);
}

template <typename T> T *from_rust_ptr(RawMut<T> t) {
  return *reinterpret_cast<T **>(&t);
}

template <typename T> RefMut<T> to_rust_ref_mut(RawMut<T> t) {
  RefMut<T> r;
  *reinterpret_cast<RawMut<T> *>(&r) = t;
  return r;
}

template <typename CppBase, typename RustDerived>
RawMut<RustDerived> RustInherit<CppBase, RustDerived>::get_rust(CppBase *base) {
  return as_rust_ptr_mut<RustDerived>((RustDerived *)(AlignedTo(
      (char *)(base) + sizeof(CppBase), DerivedAlign())));
}

template <typename CppBase, typename RustDerived>
CppBase *
RustInherit<CppBase, RustDerived>::get_base(RawMut<RustDerived> derived) {
  return reinterpret_cast<CppBase *>(AlignedTo(
      (char *)(from_rust_ptr(derived)) - sizeof(CppBase), FullAlign()));
}

template <typename CppBase, typename RustDerived>
void *RustInherit<CppBase, RustDerived>::AlignedTo(void *unaligned_addr,
                                                   size_t alignment) {
  uintptr_t aligned_addr =
      ((uintptr_t)(unaligned_addr) + alignment - 1) & ~(alignment - 1);
  return (void *)(aligned_addr);
}

template <typename CppBase, typename RustDerived>
size_t RustInherit<CppBase, RustDerived>::DerivedAlign() {
  return ::rust::__zngur_internal<RustDerived>::align_of();
}

template <typename CppBase, typename RustDerived>
size_t RustInherit<CppBase, RustDerived>::FullAlign() {
  return std::max(alignof(CppBase), DerivedAlign());
}
} // namespace rust
