mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

struct Inventory(generated::ZngurCppOpaqueOwnedObject);
struct Item(generated::ZngurCppOpaqueOwnedObject);

fn main() {
    let mut inventory = Inventory::new_empty(1000);
    inventory.add_banana(3);
    inventory.add_item(Item::new("apple", 5));
    dbg!(inventory);
}
