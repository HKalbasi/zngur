#include "generated.h"
#include <string>

using namespace rust::crate;

Inventory rust::Impl<Inventory>::new_empty(uint32_t space) {
  return Inventory(
      rust::ZngurCppOpaqueOwnedObject::build<cpp_inventory::Inventory>(space));
}

rust::Unit rust::Impl<Inventory>::add_banana(rust::RefMut<Inventory> self,
                                             uint32_t count) {
  self.cpp().add_banana(count);
  return {};
}

rust::Unit rust::Impl<Inventory>::add_item(rust::RefMut<Inventory> self,
                                           Item item) {
  self.cpp().add_item(item.cpp());
  return {};
}

Item rust::Impl<Item>::new_(rust::Ref<rust::Str> name, uint32_t size) {
  return Item(rust::ZngurCppOpaqueOwnedObject::build<cpp_inventory::Item>(
      cpp_inventory::Item{
          .name = ::std::string(reinterpret_cast<const char *>(name.as_ptr()),
                                name.len()),
          .size = size}));
}

rust::std::fmt::Result rust::Impl<Inventory, rust::std::fmt::Debug>::fmt(
    rust::Ref<::rust::crate::Inventory> self,
    rust::RefMut<::rust::std::fmt::Formatter> f) {
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
  return f.write_str(rust::Str::from_char_star(result.c_str()));
}
