#include "./generated.h"

int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
  inventory.add_banana(3);
  zngur_dbg(inventory);
}
