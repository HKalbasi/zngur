#include "generated.h"
#include <string>

namespace rust {

// Method bodies for the real C++ types. There is no rust:: mirror wrapper:
// `Impl` is specialized directly on ::cpp_inventory::Inventory/Item.
using std::ffi::CStr;
using std::fmt::Debug;
using std::fmt::Formatter;
using std::fmt::Result;

static Ref<Str> rust_str_from_c_str(const char* input) {
  return CStr::from_ptr(reinterpret_cast<const int8_t*>(input)).to_str().expect("invalid_utf8"_rs);
}

::cpp_inventory::Inventory
Impl<::cpp_inventory::Inventory>::new_empty(uint32_t space) {
  return ::cpp_inventory::Inventory(space);
}

Unit Impl<::cpp_inventory::Inventory>::add_banana(
    RefMut<::cpp_inventory::Inventory> self, uint32_t count) {
  self.cpp().add_banana(count);
  return {};
}

Unit Impl<::cpp_inventory::Inventory>::add_item(
    RefMut<::cpp_inventory::Inventory> self, ::cpp_inventory::Item item) {
  self.cpp().add_item(::std::move(item));
  return {};
}

::cpp_inventory::Item Impl<::cpp_inventory::Item>::new_(Ref<Str> name,
                                                        uint32_t size) {
  return ::cpp_inventory::Item{
      .name = ::std::string(reinterpret_cast<const char *>(name.as_ptr()),
                            name.len()),
      .size = size};
}

Result Impl<::cpp_inventory::Inventory, Debug>::fmt(
    Ref<::cpp_inventory::Inventory> self, RefMut<Formatter> f) {
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

} // namespace rust
