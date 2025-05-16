#include "generated.h"
#include <string>

using namespace rust::crate;

template <typename T> using Ref = rust::Ref<T>;
template <typename T> using RefMut = rust::RefMut<T>;

rust::Ref<rust::Str> rust_str_from_c_str(const char* input) {
  return rust::std::ffi::CStr::from_ptr(reinterpret_cast<const int8_t*>(input)).to_str().expect("invalid_utf8"_rs);
}

Inventory rust::Impl<Inventory>::new_empty(uint32_t space) {
  return Inventory(
      rust::ZngurCppOpaqueOwnedObject::build<cpp_inventory::Inventory>(space));
}

rust::Unit rust::Impl<Inventory>::add_banana(RefMut<Inventory> self,
                                             uint32_t count) {
  self.cpp().add_banana(count);
  return {};
}

rust::Unit rust::Impl<Inventory>::add_item(RefMut<Inventory> self, Item item) {
  self.cpp().add_item(item.cpp());
  return {};
}

Item rust::Impl<Item>::new_(Ref<rust::Str> name, uint32_t size) {
  return Item(rust::ZngurCppOpaqueOwnedObject::build<cpp_inventory::Item>(
      cpp_inventory::Item{
          .name = ::std::string(reinterpret_cast<const char *>(name.as_ptr()),
                                name.len()),
          .size = size}));
}

rust::std::fmt::Result rust::Impl<Inventory, rust::std::fmt::Debug>::fmt(
    Ref<Inventory> self, RefMut<rust::std::fmt::Formatter> f) {
  ::std::string result = "Inventory { remaining_space: ";
  result += ::std::to_string(self.cpp().remaining_space);
  result += ", items: [";
  bool is_first = true;
  for (const auto &item : self.cpp().items) {
    if (!is_first) {
      result += ", ";
    } else {
      is_first = false;
    }
    result += "Item { name: \"";
    result += item.name;
    result += "\", size: ";
    result += ::std::to_string(item.size);
    result += " }";
  }
  result += "] }";
  return f.write_str(rust_str_from_c_str(result.c_str()));
}
