#include "./generated.h"

int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
  inventory.add_banana(3);
  rust::Ref<rust::Str> name = rust::Str::from_char_star("apple");
  inventory.add_item(rust::crate::Item(name.to_owned(), 5));
  zngur_dbg(inventory);

  rust::std::vec::Vec<rust::crate::Item> v = inventory.into_items();
  zngur_dbg(v);
}
