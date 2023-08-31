mod generated;

#[derive(Debug)]
struct Item {
    name: String,
    size: u32,
}

#[derive(Debug)]
struct Inventory {
    items: Vec<Item>,
    remaining_space: u32,
}

impl Inventory {
    fn new_empty(space: u32) -> Self {
        Self {
            items: vec![],
            remaining_space: space,
        }
    }

    fn add_item(&mut self, item: Item) {
        self.remaining_space -= item.size;
        self.items.push(item);
    }

    fn add_banana(&mut self, count: u32) {
        for _ in 0..count {
            self.add_item(Item {
                name: "banana".to_owned(),
                size: 7,
            });
        }
    }
}
