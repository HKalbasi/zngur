#include <cstdint>
#include <string>
#include <vector>

namespace cpp_inventory {
struct Item {
  std::string name;
  uint32_t size;
};

struct Inventory {
  std::vector<Item> items;
  uint32_t remaining_space;
  Inventory(uint32_t space) : items(), remaining_space(space) {}

  void add_item(Item item) {
    remaining_space -= item.size;
    items.push_back(std::move(item));
  }

  void add_banana(uint32_t count) {
    for (uint32_t i = 0; i < count; i += 1) {
      add_item(Item{
          .name = "banana",
          .size = 7,
      });
    }
  }
};

} // namespace cpp_inventory
