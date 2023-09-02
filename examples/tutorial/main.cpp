#include "./generated.h"

int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
  inventory.add_banana(3);

  rust::std::vec::Vec<rust::crate::Item> v = inventory.into_items();
  zngur_dbg(v);
}
